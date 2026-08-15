# 从内存切换到 SQLite + SQLx

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 重启后保留目标与结果 | SQLite、SQLx、行映射、临时数据库 | 150 分钟 | `SqliteRepository` |

阶段顺序很重要：先用 `InMemoryRepository` 学共享状态和接口，再让 `SqliteRepository` 接管 SQL、migration、行转换与数据库错误。Axum 路由不需要改变。

![存储接缝：Axum 与契约测试依赖 MonitorRepository，内存和 SQLite 隐藏各自实现](../assets/architecture/repository-seam.svg)

此时已经学过 Future 与 `Send`/`Sync`，才把核心阶段的同步 trait 升级为异步存储边界：

```rust,ignore
#[async_trait]
pub trait MonitorRepository: Send + Sync {
    async fn add_target(&self, target: MonitorTarget) -> Result<StoredTarget, StoreError>;
    async fn latest_result(&self, id: i64) -> Result<Option<CheckResult>, StoreError>;
    async fn save_result(&self, id: i64, result: &CheckResult) -> Result<(), StoreError>;
}
```

`save_result` 借用结果并验证它属于同一个 target；内存和 SQLite 运行同一组契约测试，避免“可替换”只停留在类型层面。

```rust,ignore
let repository = SqliteRepository::connect("sqlite://monitor.db").await?;
let router = app(Arc::new(repository), checker);
```

测试使用 `sqlite::memory:` 且将连接池限制为一个连接。SQLite 的每个纯内存连接拥有独立数据库；如果连接池打开多个连接，可能出现“刚建的表突然不存在”的错觉。

```sh
cargo test -p monitor-store --test sqlite
cargo test -p monitor-store --test contract
cargo test -p monitor-web --test sqlite_api
```

自动测试不依赖本机已经安装 SQLite 命令行工具，也不共享开发数据库。

建表语句位于 `projects/monitor-store/migrations`，由 SQLx migrator 记录已应用版本。把 schema 从 Rust 字符串中拿出来后，数据库演进有了可审查的顺序，也不会在每个适配器方法里偷偷建表。
