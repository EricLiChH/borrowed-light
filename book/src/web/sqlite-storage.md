# 从内存切换到 SQLite + SQLx

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 重启后保留目标与结果 | SQLite、SQLx、行映射、临时数据库 | 150 分钟 | `SqliteRepository` |

阶段顺序很重要：先用 `InMemoryRepository` 学共享状态和接口，再让 `SqliteRepository` 接管 SQL、migration、行转换与数据库错误。Axum 路由不需要改变。

```rust,ignore
let repository = SqliteRepository::connect("sqlite://monitor.db").await?;
let router = app(Arc::new(repository), checker);
```

测试使用 `sqlite::memory:` 且将连接池限制为一个连接。SQLite 的每个纯内存连接拥有独立数据库；如果连接池打开多个连接，可能出现“刚建的表突然不存在”的错觉。

```sh
cargo test -p monitor-store --test sqlite
```

自动测试不依赖本机已经安装 SQLite 命令行工具，也不共享开发数据库。

建表语句位于 `projects/monitor-store/migrations`，由 SQLx migrator 记录已应用版本。把 schema 从 Rust 字符串中拿出来后，数据库演进有了可审查的顺序，也不会在每个适配器方法里偷偷建表。
