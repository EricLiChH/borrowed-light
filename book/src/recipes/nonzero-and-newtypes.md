# NonZero：让非法值无法被构造

| 元数据 | 内容 |
|---|---|
| 任务 | 消掉一个只会说“不能为零”的错误类型 |
| 概念 | newtype、`NonZeroUsize`、builder、校验该发生在哪一层 |
| 错误码 | 无——这是设计问题，编译器不会提醒你 |
| 先修 | `Result`、`Option`、`const fn` |
| 项目阶段 | 检查策略 `CheckPolicy` |

旧写法把校验放在构造函数里：

```rust,ignore
pub const fn new(timeout: Duration, attempts: usize, backoff: Duration, concurrency: usize)
    -> Result<Self, PolicyError>;
```
## 修复选择

```rust,ignore
pub struct CheckPolicy {
    total_timeout: Duration,
    max_attempts: NonZeroUsize,   // 零在这里不存在
    backoff: Backoff,
    concurrency: NonZeroUsize,
}

impl CheckPolicy {
    pub const fn builder() -> CheckPolicyBuilder;   // 具名设置，不可能写错顺序
}
```

- `PolicyError` 整个消失，`build()` 不再可能失败，调用方少一个 `?`。
- 命令行参数顺带获益：`--attempts 0` 在 `NonZeroUsize` 的 `FromStr` 那一层就被 clap 拒绝，退出码 2，压根进不了业务代码。
- 字段超过三四个、或者有多个可选字段时，就该上 builder；两个字段的结构体直接具名构造函数更省事。

## 什么时候不该这么做

- 值来自外部系统且“零”有特殊含义（例如“不限次数”）：用 `Option<NonZeroUsize>` 或专门的枚举表达，不要偷偷把零当成无限。
- 校验依赖多个字段的组合（结束时间必须晚于开始时间）：那不是单个 newtype 能表达的，仍然需要 `Result`。

决策全文见仓库的 `docs/adr/0020`。
