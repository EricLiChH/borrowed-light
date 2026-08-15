# 用 reqwest 检查本地网站

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 把目标 URL 变成检查结果 | 复用 Client、HTTP 状态、传输错误 | 120 分钟 | `HealthChecker::check` |

项目把一个 `reqwest::Client` 注入 `HealthChecker` 并反复使用。Client 内部维护连接池；每次检查重新创建 Client 会丢失复用价值，也让测试和配置分散。

```rust,ignore
let checker = HealthChecker::new(reqwest::Client::new(), CheckPolicy::default());
let result = checker.check(&target).await;
```

HTTP 500 仍然说明服务器返回了 HTTP 响应，因此当前领域结果记录为 `Reachable { status: 500 }`；DNS、连接和超时失败记录为 `Unreachable`。是否把 5xx 判为“不健康”属于更上层的产品规则，不应和“能否完成 HTTP 往返”混为一谈。

## 确定性网络测试

测试只绑定 `127.0.0.1:0`，由操作系统选择空闲端口，再返回一段最小 HTTP 响应：

```sh
cargo test -p monitor-core --test check_success
```

故意把测试服务器改成不返回响应，观察下一章如何在不真实等待两秒的情况下验证超时。
