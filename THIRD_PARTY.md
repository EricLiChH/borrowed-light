# 第三方来源与许可

本文件记录项目直接使用或明确借鉴的第三方资料。没有在仓库中复制的资源标记为“仅链接”。

| 资源 | 用途 | 使用方式 | 许可/说明 |
|---|---|---|---|
| [The Rust Programming Language](https://doc.rust-lang.org/stable/book/) | 概念准确性与错误码参考 | 仅链接，正文原创 | Rust 项目许可见上游仓库 |
| [Rustlings](https://github.com/rust-lang/rustlings) | 社区习题运行器与项目格式 | 工具依赖，不复制上游题目 | MIT |
| [mdBook](https://github.com/rust-lang/mdBook) | 静态教程构建工具 | 工具依赖 | MPL-2.0 |
| [Aquascope](https://cel.cs.brown.edu/aquascope/) | 所有权交互可视化延伸阅读 | 仅链接 | 以项目页面为准 |
| [Tokio](https://crates.io/crates/tokio)、[Axum](https://crates.io/crates/axum)、[Tower](https://crates.io/crates/tower)、[tower-http](https://crates.io/crates/tower-http)、[tracing](https://crates.io/crates/tracing) | 异步运行时、HTTP 服务、中间件与日志 | Cargo 代码依赖，不复制上游源码 | MIT；以锁文件对应版本的上游声明为准 |
| [reqwest](https://crates.io/crates/reqwest)、[SQLx](https://crates.io/crates/sqlx)、[Clap](https://crates.io/crates/clap)、[Serde](https://crates.io/crates/serde)、[Futures](https://crates.io/crates/futures)、[async-trait](https://crates.io/crates/async-trait) | HTTP 客户端、SQLite、CLI、序列化、异步组合与 trait | Cargo 代码依赖，不复制上游源码 | MIT OR Apache-2.0；以锁文件对应版本的上游声明为准 |

发布前必须逐项复核本表；许可不明确的第三方内容只链接、不复制。
