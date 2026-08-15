# SQLite 内存测试：为什么数据突然消失

| 元数据 | 内容 |
|---|---|
| 任务 | 稳定测试内存 SQLite |
| 概念 | connection、pool、temporary database |
| 先修 | SQLx、异步测试 |
| 项目阶段 | Web 持久化 |

`sqlite::memory:` 的数据库属于单个连接。测试连接池若创建多个连接，每个连接看到的是不同数据库；主线把池限制为一个连接。详见 [SQLite + SQLx](../web/sqlite-storage.md)。
