# 重返 Rust：交互式 Cookbook

面向曾经使用过 Rust、但已经很久没写的回归学习者。当前仓库包含首个 60–90 分钟样章垂直切片：诊断、所有权三视图、浏览器示例、Rustlings 练习和网站健康监测器领域模型。

## 本地运行

需要 Rust 1.88、mdBook 0.5.4 和 Rustlings 6.5.0。

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

验证项目检查点：

```sh
cargo test --workspace
```

维护者可用 `scripts/check.sh` 运行完整本地检查。源码仓库公开托管于 [GitHub](https://github.com/EricLiChH/rust-cookbook-rust)；在线教材网站和自动部署尚未启用。

## 参与贡献

提交改进前请阅读 [贡献指南](CONTRIBUTING.md) 与 [社区行为准则](CODE_OF_CONDUCT.md)。

## 许可

- 原创正文与图示：CC BY 4.0。
- 原创代码与练习：MIT OR Apache-2.0。
- 第三方资料：参见 [THIRD_PARTY.md](THIRD_PARTY.md)。
