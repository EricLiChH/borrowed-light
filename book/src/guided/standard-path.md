# 30–40 小时标准路径

这条路径为“写过 Rust、但手生了”的读者设计。时间不是阅读配额，而是预测、运行、破坏、修复和解释的实践预算。诊断已经证明熟练的部分可以跳过；每阶段的验收不能跳过。

| 阶段 | 时间 | 必做内容 | 可检查产物 |
|---|---:|---|---|
| 核心能力恢复 | 8–10 小时 | 诊断、所有权三视图、`Option`/`Result`、trait、6 个 Rustlings | 领域模型、存储接缝、全部练习通过 |
| CLI 实战 | 8–10 小时 | 参数、JSON、错误边界、顺序基线、批量检查、集成测试、安装 | `monitor` 可安装并通过离线测试 |
| 异步网络与 Web | 14–20 小时 | Future、受控并发、总超时、有限重试、错误分类、Axum、SQLite、追踪、关闭 | CLI 并发检查器与持久化 HTTP API |

```mermaid
flowchart LR
    A["诊断"] --> B["核心类型与所有权"]
    B --> C["CLI 顺序基线"]
    C --> D["Tokio + reqwest"]
    D --> E["Axum + 内存存储"]
    E --> F["SQLite + SQLx"]
    F --> G["离线全量验收"]
```

## 每个学习单元的交互循环

1. **先预测**：写下能否编译、谁拥有值、会返回哪类结果。
2. **运行**：浏览器示例负责轻量反馈，本地测试负责依赖、网络与数据库。
3. **故意弄坏**：一次只改变一个条件，让编译器或测试暴露心智模型。
4. **最小修复**：先恢复行为，再讨论更深的模块边界。
5. **迁移到项目**：把概念落到网站健康监测器，不另造一次性例子。
6. **用证据结束**：以练习、测试、命令输出或解释题为完成标准。

## 三个阶段门

### 门 1：核心能力

```sh
cd exercises
rustlings
```

完成 6 个练习，并让 `cargo test -p monitor-domain -p monitor-store` 通过。你应能解释 move 与 borrow、`Result<Option<T>, E>`、以及为什么存储 trait 是接缝而不是装饰。

### 门 2：CLI

```sh
cargo test -p monitor-cli
cargo run -p monitor-cli --bin monitor -- --help
```

完成 CLI 章节的五个实验。你应能区分参数解析、领域校验、网络调度和输出格式各自的责任。

### 门 3：异步 Web

```sh
cargo test -p monitor-core -p monitor-web
cargo run -p monitor-web
```

完成总超时、重试、并发和两种存储的离线测试，再走一遍本地 API。最后回到[毕业检查点](../web/final-checkpoint.md)回答五个解释题。

## 什么时候查配方

遇到明确错误码或局部任务时，从[配方索引](../recipes/index.md)进入；解决后回到当前阶段门。配方是同一内容的检索入口，不是另一条需要重学的课程。
