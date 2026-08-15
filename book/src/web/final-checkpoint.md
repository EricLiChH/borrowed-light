# 毕业检查点：运行完整服务

| 任务 | 预计时间 | 完成证据 |
|---|---:|---|
| 启动持久化 Web 服务并走完 API | 120 分钟 | 数据库文件、HTTP 响应、全部测试 |

启动服务：

```sh
DATABASE_URL=sqlite://monitor.db BIND_ADDRESS=127.0.0.1:3000 \
  cargo run -p monitor-web
```

服务默认输出启动、请求路径、状态码与延迟。需要调整详细程度时设置 `RUST_LOG`，例如 `RUST_LOG=monitor_web=info,tower_http=debug`。

Windows PowerShell：

```powershell
$env:DATABASE_URL = "sqlite://monitor.db"
$env:BIND_ADDRESS = "127.0.0.1:3000"
cargo run -p monitor-web
```

另开终端：

```sh
curl -X POST http://127.0.0.1:3000/targets \
  -H 'content-type: application/json' \
  -d '{"name":"Rust","url":"https://www.rust-lang.org"}'
curl -X POST http://127.0.0.1:3000/targets/1/checks
curl http://127.0.0.1:3000/targets/1/checks/latest
```

公网请求是手动演示；验收仍以离线测试为准：

```sh
scripts/check.sh
```

## 你应该能解释

1. `MonitorTarget`、`HealthChecker`、`MonitorRepository` 和 Axum Router 各自隐藏什么？
2. 为什么批量检查不能直接 `join_all` 无限启动？
3. 为什么超时、连接失败和 HTTP 500 不是同一种状态？
4. 为什么 Web 测试能替换内存与 SQLite，而不重写 handler？
5. 哪些步骤属于 Rust 语言，哪些属于 Tokio、reqwest、Axum 或 SQLx？

## 最后的破坏性实验（只针对临时数据）

1. 把 `DATABASE_URL` 改为 `sqlite::memory:`，重启后解释数据为什么消失。
2. 在 `targets_api` 中发送非法 URL，先预测状态码和 JSON 错误，再运行测试。
3. 临时让本地测试服务器不回复，确认 API 保存的是 `timeout` 分类，而不是 HTTP 500。
4. 运行服务后按 Ctrl-C，观察优雅关闭是否完成；不要用强制终止作为唯一验证。

实验数据库请使用新建的临时文件，不要指向已有数据。做完后恢复代码并运行全量检查。
