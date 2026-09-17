# 重返 Rust：交互式 Cookbook

面向曾经使用过 Rust、但已经很久没写的回归学习者。这是一条 30–40 小时的分阶段路径：先恢复核心心智模型，再完成 CLI，最后把同一个网站健康监测器升级为 Tokio + reqwest 异步客户端和 Axum + SQLite Web 服务。

学习不是纯阅读：浏览器承担短讲解、图示和可运行代码，本地承担 Rustlings、编译器反馈、离线网络测试与项目检查点。完整路线见[标准路径](book/src/guided/standard-path.md)。

## 本地运行

需要 Rust 1.94、mdBook 0.5.4 和 Rustlings 6.5.0。

```sh
cargo install mdbook --version 0.5.4 --locked
cargo install rustlings --version 6.5.0 --locked
mdbook serve book --open
```

另开一个终端运行练习：

```sh
cd exercises
rustlings
```

验证全部项目检查点：

```sh
cargo test --workspace
```

运行贯穿项目：

```sh
cargo run -p monitor-cli --bin monitor-sync -- target Rust https://www.rust-lang.org
cargo run -p monitor-cli --bin monitor -- check Rust https://www.rust-lang.org
cargo run -p monitor-web
```

维护者在 macOS/Linux 可用 `scripts/check.sh`，在 Windows PowerShell 可用 `scripts/check.ps1` 运行与 CI 对齐的完整检查。平台差异见[跨平台说明](docs/platforms.md)。

源码公开托管于 [GitHub](https://github.com/EricLiChH/borrowed-light)；在线教材网站和自动部署尚未启用。

## 参与贡献

提交改进前请阅读 [贡献指南](CONTRIBUTING.md) 与 [社区行为准则](CODE_OF_CONDUCT.md)。

## 许可

- 原创正文与图示：CC BY 4.0。
- 原创代码与练习：MIT OR Apache-2.0。
- 第三方资料：参见 [THIRD_PARTY.md](THIRD_PARTY.md)。
