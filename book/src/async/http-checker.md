# 用 reqwest 检查本地网站

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 把目标 URL 变成检查结果 | 复用 Client、HTTP 状态、传输错误 | 2.5 小时 | `HealthChecker::check` |

> **预计 2.5 小时**。上一阶段的 `monitor-sync` 用阻塞客户端逐个检查。现在才引入 Tokio，把**同一套输入输出协议**迁移到 `monitor`：单目标先保持相同行为，再让下一章加入并发窗口。

---

## 1. 换运行时，不换协议

迁移的验收标准不是「跑起来了」，而是**同样的命令、同样的 JSON 字段、同样的退出码**。三个真实样本（同一台机器、同一批本地服务器）：

```text
$ monitor-sync check local http://127.0.0.1:8733      ← 阻塞版
$ monitor check      local http://127.0.0.1:8733      ← 异步版
{
  "name": "local",
  "url": "http://127.0.0.1:8733/",
  "reachable": true,
  "status": 200,
  "failure": null,
  "reason": null
}
EXIT=0
```

协议一致带来一个很实际的收益：**CLI 阶段写的协议测试可以原样复用**。异步迁移如果改变了输出格式，那些测试立刻变红——这正是它们存在的意义。

---

## 2. Client 是资源，应该注入而不是新建

```rust,ignore
pub struct HealthChecker {
    client: reqwest::Client,
    policy: CheckPolicy,
}

impl HealthChecker {
    pub const fn new(client: reqwest::Client, policy: CheckPolicy) -> Self {
        Self { client, policy }
    }
}
```

`reqwest::Client` 内部持有**连接池**和 TLS 会话缓存。它的三个后果：

| 做法 | 后果 |
|---|---|
| 复用同一个 `Client` | 连接复用；同一主机的第二次请求省掉 TCP 握手（HTTPS 还省掉 TLS 握手） |
| 每次检查新建 `Client` | 每次重新握手；高并发时表现为端口耗尽和延迟飙升 |
| 把 `Client` 注入而不是内部创建 | 测试可以传入配置好的客户端（超时、禁代理），生产代码不用改 |

最后一条是设计层面的收益：**依赖是参数，不是全局状态。** 项目的测试全部通过 `HealthChecker::new(测试客户端, 测试策略)` 注入行为，生产代码则注入配置好的客户端。

---

## 3. 状态码的语义：这一章最重要的一条决定

```text
$ ... check local   http://127.0.0.1:8733        → reachable: true,  status: 200,  EXIT=0
$ ... check missing http://127.0.0.1:8733/nope   → reachable: true,  status: 404,  EXIT=0
$ ... check dead    http://127.0.0.1:1           → reachable: false, failure: "connect", EXIT=1
$ ... check slow    （服务器不响应）              → reachable: false, failure: "timeout", EXIT=1
```

第二行是最容易被误判的一行：**404 也是「可达」。**

领域类型把这件事写死了：

```rust,ignore
pub enum CheckOutcome {
    /// 网站返回了 HTTP 响应。
    Reachable { status: u16 },
    /// 网站无法被检查。
    Unreachable { kind: CheckFailureKind, reason: String },
}
```

`Reachable` 的含义是**「完成了一次 HTTP 往返」**，不是「一切正常」。`status` 单独带着状态码，让上层自己决定怎么解读：

| 状态码 | 说明什么 | 谁来决定是否算「不健康」 |
|---|---|---|
| 2xx | 请求成功 | 不需要 |
| 3xx | 跟随重定向后仍到达 | 调用方 |
| 4xx | 服务器收到了请求并明确拒绝 | **产品规则**（404 可能是正常的） |
| 5xx | 服务器出错了，但它回答了 | **产品规则**（通常是「不健康」） |

**为什么不在检查器里把 5xx 判成失败？** 因为「能否完成 HTTP 往返」和「这个站点健康吗」是两个问题，把后者塞进前者会让两者都无法单独改变。下一章会看到这个划分的好处：重试策略可以单独对 5xx 做处理，而领域结果仍然诚实地记录「服务器确实回了 500」。

---

## 4. 错误分类：三种失败，一个真相来源

状态码之外的失败按传输层原因分类：

```rust,ignore
/// 稳定类别：数据库、HTTP API 和退出码都依赖它。
pub fn classify_transport_error(error: &reqwest::Error) -> CheckFailureKind {
    if error.is_timeout() {
        CheckFailureKind::Timeout
    } else if error.is_connect() {
        CheckFailureKind::Connect
    } else {
        CheckFailureKind::Request
    }
}
```

| 类别 | 典型原因 | 该怎么做 |
|---|---|---|
| `timeout` | 服务器迟迟不响应，或总期限到了 | 可能只是慢；值得重试 |
| `connect` | DNS 解析失败、连接被拒、TLS 握手失败 | 多半是配置或网络问题；重试意义有限 |
| `request` | 协议层错误、重定向过多、响应体读取失败 | 看具体原因 |

关键在「稳定类别」四个字：这个字符串会出现在**数据库列、HTTP 响应字段和 CLI 输出**里（`docs/adr/0019`）。所以它的拼写是 API 的一部分，而不是调试信息——这就是为什么它由 `CheckFailureKind::as_str` 一个地方统一定义，而不是在每个 crate 里各写一遍 `match`。
---

## 5. 确定性网络测试

测试只绑定 `127.0.0.1:0`，由操作系统选择空闲端口，再返回一段最小 HTTP 响应：

```rust,ignore
pub async fn serve_once(response: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("测试服务器应该能绑定");
    let address = listener.local_addr().expect("监听器应该有一个地址");

    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("应该接受连接");
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request).await;
        let _ = stream.write_all(response.as_bytes()).await;
    });

    format!("http://{address}")
}
```

三个决定：

| 决定 | 理由 |
|---|---|
| 端口用 `0` | 让内核分配，测试之间不会抢端口，也不依赖固定的 3000/8080 |
| 手写最小 HTTP 响应，不用 mock 框架 | 响应只有 40 来个字节，直接控制状态码、头、以及「不回响应」这种极端情况 |
| 只在 `127.0.0.1` 上通信 | 不触网、不需要证书、CI 里也能跑 |

那 40 个字节长这样：

```text
HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n
HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n
HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n
```

**能直接写出「不回响应」的服务器**是手写测试服务器最大的价值：把 `accept` 之后的 `write_all` 去掉，就得到一个永远不回答的服务端——下一章的超时测试正是靠它，在不真实等待的前提下验证总期限。

```sh
cargo test -p monitor-core --test check_success
cargo test -p monitor-cli --test check_command
```

---

## 6. 深水区

### 6.1 一次「检查」可能发出 11 个请求

`reqwest` 默认跟随重定向，上限 10 次。所以「检查一个 URL」在网络层面可能是一个原始请求加十次重定向。项目的重定向测试把这个事实写成了断言：

```rust,ignore
// 一次尝试最多 11 个请求：原始请求 + reqwest 的 10 次重定向
let requests = requests.load(Ordering::SeqCst);
assert!((2..=11).contains(&requests));
```

它同时验证了另一件事：**重定向过多属于 `E0502` 意义上的「确定性失败」，不该重试。** 重试它只会把同一个错误重复三遍，并把延迟变成三倍。这就是 `is_retryable_transport_error` 里排除 `is_redirect()` 的原因。

### 6.2 代理：环境变量会改变你测的对象

上一章已经演示过：本机地址的请求被代理接走，报告出假的 `reachable: true`。健康监测器尤其要小心这一点——**它可能一直在测代理的可用性。**

处理方式有两种，各有适用场景：

| 场景 | 做法 |
|---|---|
| 本地地址（`127.0.0.1`、内网） | 必须绕过代理：`.no_proxy()` |
| 公网站点 | 保留系统代理设置，但要在文档里写明「结果包含代理的健康状况」 |

### 6.3 超时有两层，别只设一层

```rust,ignore
// 第一层：reqwest 自己的超时（单次请求）
let client = reqwest::Client::builder().timeout(Duration::from_secs(5)).build()?;

// 第二层：整个检查的总期限（覆盖重试 + 退避）
tokio::time::timeout(policy.total_timeout(), attempt_until_settled(target)).await
```

只有第一层时，「两次重试各超时 5 秒 + 退避」会让一次检查花掉十几秒；只有第二层时，单次请求可能把整个预算耗光，导致重试永远不会发生。两层配合的语义是下一章的主题。

### 6.4 HEAD 还是 GET

健康检查常用 `HEAD` 因为它不传响应体、更省流量。但要注意：不少服务器不支持 `HEAD`，或者返回与 `GET` 不同的状态码（例如同一路径 `GET` 返回 405 而 `HEAD` 返回 200）。**先用 `GET` 把基线跑对，再把 `HEAD` 当成一个需要单独验证的优化。**

---

## 7. 常见误解

| 误解 | 准确说法 |
|---|---|
| 「404 说明站点健康检查失败」 | 404 是「完成了一次 HTTP 往返」，属于 `Reachable`；是否算故障是上层规则。 |
| 「连接失败和超时是一回事」 | 分类不同：`connect` 多半是配置问题，`timeout` 可能只是慢。重试策略也不同。 |
| 「每次请求新建 Client 更安全」 | 会丢掉连接池和 TLS 会话；应复用一个 Client 并注入使用方。 |
| 「测试要连真实网站才算数」 | 真实网站会引入网络抖动、限流和不可重复的状态；本地临时端口能覆盖全部协议分支。 |
| 「设了 `Client` 的超时就不用管总期限」 | 重试与退避会把单次超时累加；必须另有总期限兜底。 |

---

## 8. 练习与自测

### 练习

```sh
cargo test -p monitor-core --test check_success
cargo test -p monitor-core --test retry
```

然后做这个实验：**把测试服务器改成不返回响应**，观察下一章如何在不真实等待两秒的情况下验证超时（提示：`tokio::time::paused`）。

### 自测清单（能全部做到才算掌握）

1. 说明为什么 `404` 被记录为 `Reachable`，以及这条划分带来了什么好处。
2. 说出复用 `reqwest::Client` 的两个具体收益。
3. 给出三种失败类别各自的典型原因，并说明为什么它们必须是稳定字符串。
4. 用 `127.0.0.1:0` 写一个只服务一次的异步测试服务器，并说明端口为什么用 0。
5. 解释一次「检查」为什么可能产生 11 个请求，以及本项目为什么不重试这种失败。
6. 说出两层超时各自的职责，以及只有一层时会出什么问题。
7. 解释代理如何让健康检查测错对象，并说出两种处理方式。
8. 给出「先 GET 再考虑 HEAD」的理由。

---

## AI 辅导提示词

这三段可以直接复制给 AI 助手（Kimi、ChatGPT 等）。它们的设计意图是**让助手出题和追问，而不是替你写代码**——完整方法论见[用 AI 助手当教练](../guided/ai-tutor.md)。

```text
下面是我的检查函数和它的错误分类。请回答三个问题：
1. 哪些失败被我归成了一类，但其实调用方的处理方式不同？
2. 有没有哪一类失败重试是纯浪费？
3. 分类的字符串会不会泄漏到对外协议里，改名的代价是什么？
只给分析，不要改代码。

[粘贴检查函数]
```

```text
请出 3 道判断题，考察「可达」与「健康」的区别：
给定状态码、超时、连接失败等几种情况，让我判断该记成 Reachable 还是 Unreachable，
以及退出码该是 0 还是 1。我答完后逐条点评。
```

```text
我要为这个检查器写确定性测试。请先问我三个问题
（哪些分支需要覆盖、哪些需要真实计时、哪些需要控制并发时序），
再给出测试清单和每个测试用的手段（临时端口、暂停时钟、信号量）。
不要直接给测试代码。
```
