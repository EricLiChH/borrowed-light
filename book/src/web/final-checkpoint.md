# 毕业检查点：运行完整服务

| 任务 | 预计时间 | 完成证据 |
|---|---:|---|
| 启动持久化 Web 服务并走完 API | 2 小时 | 数据库文件、HTTP 响应、全部测试 |

> **预计 2 小时**。「毕业」不是感觉，是**证据**。这一章要求你端到端跑一遍，并回答五个解释题——答不上来的地方，就是还没毕业的地方。

---

## 1. 端到端演示：一次真实的毕业跑

启动服务：

```sh
DATABASE_URL=sqlite://monitor.db BIND_ADDRESS=127.0.0.1:3011 \
  cargo run -p monitor-web
```

服务默认输出启动、请求路径、状态码与延迟。需要调整详细程度时设置 `RUST_LOG`，例如 `RUST_LOG=monitor_web=info,tower_http=debug`。

Windows PowerShell：

```powershell
$env:DATABASE_URL = "sqlite://monitor.db"
$env:BIND_ADDRESS = "127.0.0.1:3011"
cargo run -p monitor-web
```

另开一个终端，走完四条请求（下面是真实输出）：

```text
$ curl -i -X POST http://127.0.0.1:3011/targets \
       -H 'content-type: application/json' \
       -d '{"name":"self","url":"http://127.0.0.1:3011/healthz"}'
HTTP/1.1 201 Created
{"id":1,"name":"self","url":"http://127.0.0.1:3011/healthz"}

$ curl -X POST http://127.0.0.1:3011/targets/1/checks
{"target_id":1,...,"reachable":true,"status":204,"failure":null,"reason":null}

$ curl http://127.0.0.1:3011/targets/1/checks/latest
{"target_id":1,...,"reachable":true,"status":204,"failure":null,"reason":null}

$ curl -s -w '\nSTATUS=%{http_code}\n' http://127.0.0.1:3011/targets/99
{"code":"not_found","error":"target 99 was not found"}
STATUS=404
```

然后**去看数据库**——这一步最容易被跳过，但它才是「持久化」的证明：

```text
$ sqlite3 monitor.db 'SELECT * FROM targets; SELECT * FROM check_results;'
1|self|http://127.0.0.1:3011/healthz
1|1|204||

$ sqlite3 monitor.db 'PRAGMA journal_mode;'
wal

$ ls monitor.db*
monitor.db  monitor.db-shm  monitor.db-wal
```

最后按 `Ctrl-C` 观察优雅关闭：正在处理的请求会跑完，只是不再接受新的连接。**不要用强制终止作为唯一验证**——那样你无法区分「优雅关闭生效」和「进程被杀掉」。

公网请求（例如 `https://www.rust-lang.org`）是手动演示；验收仍以离线测试为准。

---

## 2. 验收清单

按顺序跑完，全部通过才算毕业：

```sh
# 1. 全量检查（格式、clippy、测试、链接、书、练习）
scripts/check.sh

# 2. 分项确认关键的几组
cargo test -p monitor-domain          # 领域不变量
cargo test -p monitor-store           # 两个适配器 + 契约测试
cargo test -p monitor-core            # 并发、超时、重试
cargo test -p monitor-web             # HTTP 边界与错误语义
cargo test -p monitor-cli             # 进程边界：退出码与输出协议
```

每一项对应的能力：

| 命令 | 它在证明什么 |
|---|---|
| `cargo test -p monitor-domain` | 你能让非法状态表达不出来 |
| `cargo test -p monitor-store --test contract` | 你的接缝是真的（两个实现表现一致） |
| `cargo test -p monitor-core --test timeout` | 你理解总期限与重试的关系 |
| `cargo test -p monitor-web --test error_semantics` | 你能把领域错误翻译成正确的状态码 |
| `cargo test -p monitor-cli --test exit_codes` | 你区分了「检查失败」与「命令失败」 |

**如果某组测试需要你改代码才能过，那说明这一章对应的能力还没建立**，回到那一章，而不是改测试。
---

## 3. 五个解释题

**先自己写答案，再看要点。** 这五题覆盖的是「离开教材之后你还能不能干活」，而不是记忆。

### 1. 四个模块各自隐藏什么？

<details><summary>参考答案要点</summary>

| 模块 | 隐藏的复杂度 | 对外的承诺 |
|---|---|---|
| `MonitorTarget` | URL 解析与规范化、名字与 URL 的不变量 | 构造成功即合法 |
| `HealthChecker` | 并发窗口、总期限、重试与退避、失败分类 | 返回一个 `CheckResult`，永不失败 |
| `MonitorRepository` | 内存锁 / SQL / migration / 行映射 | 一组与实现无关的异步操作 |
| Axum Router | HTTP 方法、路径、extractor、状态码翻译 | 一组 JSON 端点 |

关键判断：**每个模块的对外接口是否比内部实现小得多？** 如果某层的公开 API 和实现一样复杂，它就没有隐藏任何东西。

</details>

### 2. 为什么批量检查不能直接 `join_all` 无限启动？

<details><summary>参考答案要点</summary>

三个后果：**本机资源**（一批 1000 个目标就是 1000 个并发连接，可能耗尽端口与文件描述符）；**对端**（很多站点会限流甚至封禁，你会把一个监控工具变成压测工具）；**超时判定失真**（资源争抢让所有请求都变慢，超时不再反映站点本身的问题）。

正确做法是 `buffered(concurrency)`，让并发数成为一个**容量参数**——和 channel 容量、连接池大小同类。

</details>

### 3. 为什么超时、连接失败和 HTTP 500 不是同一种状态？

<details><summary>参考答案要点</summary>

- **HTTP 500 是「可达」**：服务器确实完成了一次 HTTP 往返。是否算「不健康」是上层的产品规则。
- **超时与连接失败是「不可达」**，但分类不同：`timeout` 可能只是慢（值得重试），`connect` 多半是配置或网络问题（重试意义有限）。

把它们混成一种状态，会让三件事同时做不到：调用方无法区分该重试还是该报警；统计无法按原因聚合；测试无法断言具体行为。

</details>

### 4. 为什么 Web 测试能替换内存与 SQLite，而不重写 handler？

<details><summary>参考答案要点</summary>

因为 `app` 对 `R: MonitorRepository` 泛型，状态通过 `with_state` 注入。handler 只依赖 trait 提供的能力，不知道背后是 `HashMap` 还是 SQLite。

前提是**契约测试**：`tests/contract.rs` 用同一组断言跑遍所有实现。没有它，「可替换」只是类型签名上的说法。

</details>

### 5. 哪些属于 Rust 语言，哪些属于生态？

<details><summary>参考答案要点</summary>

| 属于 Rust 语言 | 属于生态 |
|---|---|
| 所有权、借用、生命周期 | Tokio 的任务与运行时 |
| `trait`、泛型、`impl Trait` | reqwest 的 Client 与错误分类 |
| `Result`/`Option` 与 `?` | Axum 的 Router 与 extractor |
| `Send`/`Sync` 与线程 | SQLx 的查询、迁移与连接池 |
| `Future` trait 与 `async`/`await` 语法 | 具体运行时的调度策略 |

**这条区分很实用**：语言层的知识会跟着你很多年，生态 API 会变。学的时候把注意力放在左边，查阅的时候接受右边会过时。

</details>

---

## 4. 最后的破坏性实验（只针对临时数据）

每个实验都先**预测**再运行：

1. 把 `DATABASE_URL` 改为 `sqlite::memory:`，重启后解释数据为什么消失。
2. 在 `targets_api` 中发送非法 URL，先预测状态码和 JSON 错误，再运行测试。
3. 临时让本地测试服务器不回复，确认 API 保存的是 `timeout` 分类，而不是 HTTP 500。
4. 运行服务后按 Ctrl-C，观察优雅关闭是否完成；不要用强制终止作为唯一验证。
5. **把并发数改成目标总数**，观察「更快」是否真的更快，以及失败率如何变化。

实验数据库请使用新建的临时文件，不要指向已有数据。做完后恢复代码并运行全量检查。

---

## 5. 怎么知道你真的会了

三个阶段的自测表。**能全部做到才算毕业**，任何一条做不到都指向一个具体章节。

### 语言层

| 能力 | 对应章节 |
|---|---|
| 预测一段代码是否编译通过，并说出错误码 | [所有权](../guided/ownership.md) |
| 解释 `String` 的移动在机器层面发生了什么 | [所有权](../guided/ownership.md) |
| 用 `enum` 让非法状态无法表示 | [值、枚举与集合](../core/values-collections-modules.md) |
| 说出 `Copy` 与 `Clone` 的两个区别 | [所有权](../guided/ownership.md) |
| 写出 `?` 的等价 `match` 形式 | [Option 与 Result](../core/result-option.md) |
| 遇到生命周期报错时判断「加标注还是改设计」 | [生命周期、泛型与闭包](../core/lifetimes-generics-closures.md) |
| 解释 `Rc<RefCell<T>>` 为什么不能跨线程 | [线程](../async/threads-channels-state.md) |
| 解释创建 future 为什么什么都不做 | [Future 与运行时](../async/future-runtime.md) |

### 工程层

| 能力 | 对应章节 |
|---|---|
| 为一个 trait 写契约测试，新增实现只加三行 | [trait 与测试](../core/traits-and-tests.md) |
| 设计错误类型：按层次选择枚举 / `Box<dyn Error>` | [Option 与 Result](../core/result-option.md) |
| 用退出码区分「检查失败」与「命令失败」 | [CLI](../cli/monitor-cli.md) |
| 写下界并估算一批目标的最坏耗时 | [受控并发](../async/concurrency-retry.md) |
| 把领域错误映射成正确的 HTTP 状态码 | [Axum](../web/axum-api.md) |
| 写只增不改的迁移，并解释校验和的作用 | [SQLite](../web/sqlite-storage.md) |

### 调试层

| 能力 | 对应章节 |
|---|---|
| 读懂 `E0382`/`E0502`/`E0499` 并判断删哪一行能过 | [配方索引](../recipes/index.md) |
| 区分「编译器的建议」与「我真正想要的」 | [Option 与 Result](../core/result-option.md) |
| 用 `127.0.0.1:0` 写确定性网络测试 | [trait 与测试](../core/traits-and-tests.md) |
| 用暂停时钟验证超时，而不是 `sleep` | [受控并发](../async/concurrency-retry.md) |
| 分辨「必须 sleep 才能过」是设计问题还是测试问题 | [trait 与测试](../core/traits-and-tests.md) |

---

## 6. 毕业之后

三个方向，按兴趣选：

1. **把项目做真**：加 `DELETE /targets/{id}`、加历史查询、加 `Retry-After` 的完整支持、给未处理的路径补 `CHECK` 约束。
2. **补上没学的那部分**：宏（`macro_rules!`、过程宏）、`unsafe` 与 FFI、性能剖析——这些不在标准路径里，因为回归学习者先用不到。
3. **换一个领域重做一遍**：把「检查 URL」换成解析器、聊天服务或 CLI 工具。**同一套分层（领域 / 核心 / 存储 / 边界）会再做一遍，而这次你会先画出边界，而不是事后重构。**

继续的路线见[三个毕业挑战与方向路线](../guided/next-projects.md)。
