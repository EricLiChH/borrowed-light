# Axum `State`：共享状态放在哪里

| 元数据 | 内容 |
|---|---|
| 任务 | 在 handler 间共享线程安全依赖 |
| 概念 | `State`、`Arc`、trait object |
| 先修 | trait、async、所有权 |
| 项目阶段 | Web 服务 |

把长期存在的 repository 和 checker 放进 Router state；不要在每个 handler 中重新创建 Client 或数据库池。规范实现见 [Axum API](../web/axum-api.md)。
