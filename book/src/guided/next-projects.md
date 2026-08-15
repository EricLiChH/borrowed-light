# 三个毕业挑战与方向路线

主线完成后任选一个挑战；不要同时开三个坑。每个项目先复用本书的阶段门、离线测试和错误分类，再学习领域特有知识。

## 挑战 A：PNG 隐写 CLI

- 练习：字节切片、二进制格式、校验、文件错误。
- 最小产物：列出/写入/删除自定义 chunk，所有测试使用临时文件。
- 参考设计：[PNGme](https://jrdngr.github.io/pngme_book/)（仅链接，题目自行重写）。

## 挑战 B：mini-redis 子集

- 练习：TCP frame、task、channel、共享状态、优雅关闭。
- 最小产物：只实现 `GET`/`SET` 和一个有界 manager channel。
- 起点：[Tokio tutorial](https://tokio.rs/tokio/tutorial)。

## 挑战 C：文字冒险状态机

- 练习：enum、模式匹配、所有权、序列化、确定性随机源。
- 最小产物：5 个场景、可保存进度、状态迁移属性测试。
- 约束：不引入 Web 框架，证明领域模型可以独立测试。

## 方向路线

| 方向 | 下一项先修 | 一手入口 |
|---|---|---|
| 后端 | 事务、认证、可观测性 | [Axum 文档](https://docs.rs/axum)、[SQLx 文档](https://docs.rs/sqlx) |
| CLI / DevTools | 流式 I/O、信号、发布 | [Command Line Applications in Rust](https://rust-cli.github.io/book/) |
| WebAssembly | 所有权边界、JS 互操作 | [Rust and WebAssembly](https://rustwasm.github.io/docs/book/) |
| 嵌入式 | `no_std`、内存映射、外设 | [Embedded Rust Book](https://docs.rust-embedded.org/book/) |
| 系统 / OS | unsafe 边界、布局、并发原语 | [Rustonomicon](https://doc.rust-lang.org/nomicon/) |

路线表不是待办清单。先选一个真实问题，再补它要求的那一列知识。
