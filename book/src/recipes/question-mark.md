# `?`：传播错误而不丢失控制权

| 元数据 | 内容 |
|---|---|
| 任务 | 在可失败步骤之间提前返回错误 |
| 概念 | `Result`、`From`、error propagation |
| 先修 | enum、模式匹配 |
| 项目阶段 | 核心能力与 CLI |

`?` 会在成功时取出值，在失败时把可转换的错误返回给调用者。规范讲解和练习见 [`Option` 与 `Result`](../core/result-option.md)。
