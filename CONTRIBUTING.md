# 贡献指南

感谢你帮助改进《重返 Rust：交互式 Cookbook》。本项目欢迎错别字修正、概念澄清、可复现的错误报告、练习改进和新的原创配方。

## 开始之前

1. 先搜索现有 issue，避免重复工作。
2. 较大的目录调整、技术选型或新阶段内容，请先开 issue 说明目标与范围。
3. 不要复制许可不明确的教程、题目、图片或大段文字；只提交你有权授权的原创内容。

## 本地验证

项目固定使用 Rust 1.94、mdBook 0.5.4 和 Rustlings 6.5.0。

```sh
cargo install mdbook --version 0.5.4 --locked
cargo install rustlings --version 6.5.0 --locked
scripts/check.sh
```

Windows PowerShell 使用：

```powershell
./scripts/check.ps1
```

完整平台说明见 [`docs/platforms.md`](docs/platforms.md)。

提交前请确认格式、Clippy、Cargo 测试、mdBook 测试/构建以及 Rustlings 练习与参考解全部通过。

## 内容约定

- 中文主讲，关键术语首次出现时补充英文。
- 困难概念应包含预测、可运行示例、失败实验、文字说明和不依赖颜色的图示。
- 引导路径与配方索引共用规范正文，避免复制两份教程内容。
- 自动测试不得依赖公网、真实等待或共享数据库。
- 新增第三方参考时同步更新 `THIRD_PARTY.md`。

## 许可

提交正文或图示，即表示你同意按 CC BY 4.0 提供该贡献；提交代码或练习，即表示你同意按 MIT OR Apache-2.0 提供该贡献。

参与项目时请遵守 `CODE_OF_CONDUCT.md`。
