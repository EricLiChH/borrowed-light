# 如何使用配方

配方与引导路径使用同一批概念，不复制另一套教程。遇到具体问题时按以下入口查找：

| 信号 | 配方 |
|---|---|
| `borrow of moved value`、`E0382` | [值已被移动](e0382-moved-value.md) |
| `cannot borrow ... as mutable`、`E0502` | [借用发生冲突](e0502-borrow-conflict.md) |
| `?` 无法转换错误 | [传播错误而不丢上下文](question-mark.md) |
| future is not `Send` | [`Send` / `Sync`](send-sync.md) |
| Axum handler 需要共享数据 | [Axum State](axum-state.md) |
| SQLite 内存表偶尔不存在 | [SQLite 内存测试](sqlite-memory.md) |

每个配方都按“意图 → 最小失败 → 判断问题 → 修复选择”组织。修复目标不是让编译器闭嘴，而是让接口表达真实所有权意图。
