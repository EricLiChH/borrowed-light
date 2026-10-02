# 从内存切换到 SQLite + SQLx

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 重启后保留目标与结果 | SQLite、SQLx、行映射、临时数据库 | 3 小时 | `SqliteRepository` |

> **预计 3 小时**。这一章把存储从一个 `HashMap` 换成真正的数据库。最值得注意的是**没有变的东西**：领域层、检查器、HTTP handler 一行都没改。

---

## 1. 阶段顺序：为什么先写内存版

先用 `InMemoryRepository` 学共享状态和接口，再让 `SqliteRepository` 接管 SQL、migration、行转换与数据库错误。

![存储接缝：Axum 与契约测试依赖 MonitorRepository，内存和 SQLite 隐藏各自实现](../assets/architecture/repository-seam.svg)

这个顺序的收益在**测试策略**上最明显：内存版让契约测试跑得飞快，而 SQLite 版只需要证明「同一组断言在真实数据库上也成立」。

```sh
cargo test -p monitor-store --test contract      # 同一组断言，两个实现
cargo test -p monitor-store --test in_memory     # 快
cargo test -p monitor-store --test sqlite        # 真实数据库
```

**如果没有契约测试，内存版就只是「另一份实现」**，而不是「可以被替换的接口」。

---

## 2. 异步 trait 的形状

```rust,ignore
// 实现方照旧写 async fn；trait 里声明成 impl Future + Send，
// 于是既不装箱，axum 的 handler 又能满足 Send。
pub trait MonitorRepository: Send + Sync {
    fn add_target(&self, target: MonitorTarget)
        -> impl Future<Output = Result<StoredTarget, StoreError>> + Send;
    fn latest_result(&self, id: i64)
        -> impl Future<Output = Result<Option<CheckResult>, StoreError>> + Send;
    fn save_result(&self, id: i64, result: &CheckResult)
        -> impl Future<Output = Result<(), StoreError>> + Send;
}
```

异步 trait 方法有三种写法，选哪种不是风格问题：

| 写法 | 装箱 | 能用于 `dyn` | axum handler 可用 |
|---|---|---|---|
| `async fn` in trait | 否 | 否 | 否（future 不保证 `Send`） |
| `#[async_trait]` | 每次调用一次 | 是 | 是 |
| `-> impl Future + Send` | 否 | 否 | 是 |

本项目选第三种，代价是放弃 `dyn`——所以 `app` 改成对 `R: MonitorRepository` 泛型，`with_state` 再把 `R` 擦除。完整推导见 `docs/adr/0017`。

`save_result` 借用结果并验证它属于同一个 target；内存和 SQLite 运行同一组契约测试，避免「可替换」只停留在类型层面。

---

## 3. 真实的数据库长什么样

启动过一次服务之后，磁盘上会留下三个文件：

```text
$ ls -la /tmp/monitor-demo.db*
-rw-r--r--  monitor-demo.db         ← 主数据库
-rw-r--r--  monitor-demo.db-shm     ← 共享内存索引（WAL 模式）
-rw-r--r--  monitor-demo.db-wal     ← 预写日志（WAL 模式）
```

后两个文件是**开启 WAL 的证据**。它们不是垃圾文件，删掉会让未落盘的写入丢失。

schema 由迁移创建：

```text
$ sqlite3 /tmp/monitor-demo.db '.schema'
CREATE TABLE targets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    url TEXT NOT NULL
);
CREATE TABLE check_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_id INTEGER NOT NULL REFERENCES targets(id),
    status INTEGER,
    failure_kind TEXT,
    reason TEXT
);
CREATE INDEX check_results_target_id_id
    ON check_results (target_id, id DESC);
CREATE TABLE _sqlx_migrations ( ... );
```

真实数据（就是上一章那次自检留下的）：

```text
$ sqlite3 /tmp/monitor-demo.db 'SELECT * FROM targets; SELECT * FROM check_results;'
1|self|http://127.0.0.1:3011/healthz
1|1|204||
```

### 三处设计值得停留

**1. `check_results` 用「可空列组合」编码一个枚举。**

```text
status=204, failure_kind=NULL, reason=NULL   → Reachable { status: 204 }
status=NULL, failure_kind='connect', ...     → Unreachable { kind: Connect, ... }
```

它其实有四种位组合，其中「`status` 和 `failure_kind` 同时为 NULL」这种是非法状态。数据库层没有办法表达「恰好一个非空」——这是**关系模型和枚举类型之间的阻抗失配**，本章第 9 节会讲怎么处理它。

**2. `id INTEGER PRIMARY KEY AUTOINCREMENT` 同时承担两个角色**：唯一标识，以及**时间顺序**。`latest_result` 用 `ORDER BY id DESC` 取最新，而不是比时间戳——因为自增整数不会有「同一秒内两条记录谁先谁后」的歧义。

**3. 索引 `(target_id, id DESC)` 是为 `latest_result` 准备的。** 那个查询的形状是「按 target_id 过滤，按 id 倒序取第一条」，复合索引正好覆盖它。

---

## 4. 行映射：从 SQL 行到领域值

```rust,ignore
#[derive(Debug, sqlx::FromRow)]
struct TargetRow {
    id: i64,
    name: String,
    url: String,
}

impl TryFrom<TargetRow> for StoredTarget {
    type Error = StoreError;

    fn try_from(row: TargetRow) -> Result<Self, Self::Error> {
        // 关键：数据库里的字符串必须重新过一遍领域校验
        let target = MonitorTarget::new(row.name, row.url).map_err(StoreError::corrupt)?;
        Ok(Self::new(row.id, target))
    }
}
```

两个要点：

- `#[derive(FromRow)]` 让「列名 ↔ 字段名」的对应关系由 derive 完成，省掉一长串 `try_get`。它同时也是一份**文档**：看结构体就知道表里有哪些列。
- **`MonitorTarget::new` 在这里再跑一次**，不是多余。数据库里的行可能是旧版本程序写的、或者被人手工改过。把行变回领域值必须重新验证，失败就归类为 `Corrupt`——而不是让一个非法的 URL 流进系统。

这条规则可以推广：**信任边界上的每一次「反序列化」都要重新校验。** 数据库、JSON 请求体、配置文件都一样。

---

## 5. 写路径：一个事务里做完两件事

```rust,ignore
async fn save_result(&self, target_id: i64, result: &CheckResult) -> Result<(), StoreError> {
    // 验证目标与写入结果必须原子完成：
    // 分成两条独立语句会留下「检查完、还没写」的窗口。
    let mut transaction = self.pool.begin().await?;

    let row = sqlx::query_as::<_, TargetRow>("SELECT id, name, url FROM targets WHERE id = ?")
        .bind(target_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(StoreError::NotFound { target_id })?;

    let stored = StoredTarget::try_from(row)?;
    if stored.target() != result.target() {
        return Err(StoreError::TargetMismatch { target_id });
    }

    let (status, kind, reason) = outcome_columns(result.outcome());
    sqlx::query(
        "INSERT INTO check_results (target_id, status, failure_kind, reason) VALUES (?, ?, ?, ?)",
    )
    .bind(target_id)
    .bind(status)
    .bind(kind)
    .bind(reason)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    Ok(())
}
```

这里有**两层保护，职责不同**：

| 保护 | 防的是什么 | 属于哪一层 |
|---|---|---|
| 事务 | 检查通过之后、写入之前目标被删掉的竞态 | 存储实现 |
| `TargetMismatch` 校验 | 结果被存到了错误的目标上（调用方的逻辑错误） | 领域不变量 |

第一层是技术性的，第二层是语义性的。**只有事务没有校验**，会把数据写脏；**只有校验没有事务**，会在并发下偶尔失败。

### 早退时事务会怎样

`return Err(...)` 时，`transaction` 被 `drop`，SQLx 自动回滚。这是「`Drop` 是清理钩子」的一个好例子：**你不需要写 `rollback()`**，只需要确保没有走到 `commit()`。

### `RETURNING` 省掉一次往返

```rust,ignore
let row = sqlx::query("INSERT INTO targets (name, url) VALUES (?, ?) RETURNING id")
    .bind(target.name())
    .bind(target.url_str())
    .fetch_one(&self.pool)
    .await?;
let id = row.try_get::<i64, _>("id")?;
```

以前要写 `INSERT` 再 `SELECT last_insert_rowid()` 两条语句，中间还可能被别的写入插队。`RETURNING` 让「插入并拿到 id」变成一次原子往返。
---

## 6. 错误也要有类型

`StoreError` 曾经是一个装着 `String` 的结构体，于是「目标不存在」和「数据库坏了」在 HTTP 层都变成 `500`。现在它是枚举，Web 层匹配变体决定状态码：

| 变体 | 含义 | HTTP |
|---|---|---|
| `NotFound { target_id }` | 没有这个目标 | 404 |
| `TargetMismatch { target_id }` | 结果不属于这个目标 | 409 |
| `Corrupt { detail }` | 存进去的行读不回来 | 500 |
| `Migration { detail }` | 建表或升级失败 | 500 |
| `Database(sqlx::Error)` | 数据库本身报错 | 500 |

`Database` 通过 `Error::source` 保留底层原因，所以 `eprintln!("{error}")` 仍然能看到 SQLite 的原始报错：

```rust,ignore
impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),   // 保留因果链
            Self::NotFound { .. }
            | Self::TargetMismatch { .. }
            | Self::Corrupt { .. }
            | Self::Migration { .. } => None,
        }
    }
}
```

**「能不能 `match`」和「能不能看到原因」是两件事**，枚举变体解决前者，`source()` 解决后者。只做前者会丢失排障信息，只做后者会让调用方只能解析字符串。

---

## 7. 连接池的两种形状

`SqliteRepository::connect` 会自动选连接池大小，调用方不需要记住这条规则：

```rust,ignore
let options = SqliteConnectOptions::from_str(database_url)?
    .create_if_missing(true)
    .foreign_keys(true)
    .busy_timeout(BUSY_TIMEOUT);

// 内存库活在单个连接里；文件库可以并发读，并开启 WAL。
let in_memory = options.get_filename().to_string_lossy() == ":memory:";
let options = if in_memory { options } else { options.journal_mode(SqliteJournalMode::Wal) };
let max_connections = if in_memory { 1 } else { FILE_POOL_SIZE };
```

三条规则与它们的原因：

| 规则 | 原因 |
|---|---|
| 内存库固定 1 个连接 | **SQLite 的每个纯内存连接都是一个独立数据库**。池子开到两个，就会出现「刚建的表突然不存在」的错觉 |
| 文件库开启 WAL | 写前日志让读不阻塞写、写不阻塞读（第 9 节展开） |
| `busy_timeout(5s)` | 遇到写锁时等一会儿，而不是立刻返回 `SQLITE_BUSY` |

WAL 生效是有据可查的——直接问数据库：

```text
$ sqlite3 /tmp/monitor-demo.db 'PRAGMA journal_mode;'
wal
```

以及磁盘上那个 `-wal` 文件（第 3 节）。

```rust,ignore
let repository = SqliteRepository::connect("sqlite://monitor.db").await?;
let router = app(Arc::new(repository), checker);
```

```sh
cargo test -p monitor-store --test sqlite
cargo test -p monitor-store --test contract
cargo test -p monitor-store --test file_database
cargo test -p monitor-web --test sqlite_api
```

**自动测试不依赖本机已安装 SQLite 命令行工具，也不共享开发数据库**：内存库用于快速契约测试，文件库测试用临时文件并在析构时连同 `-wal`、`-shm` 一起删除。这是 `docs/adr/0011` 的「确定性测试环境」在存储层的具体执行。

---

## 8. 迁移：schema 也是代码

```sql
-- projects/monitor-store/migrations/0001_initial.sql
CREATE TABLE targets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    url TEXT NOT NULL
);

CREATE TABLE check_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_id INTEGER NOT NULL REFERENCES targets(id),
    status INTEGER,
    failure_kind TEXT,
    reason TEXT
);

CREATE INDEX check_results_target_id_id
    ON check_results (target_id, id DESC);
```

它由编译期嵌入的 migrator 应用：

```rust,ignore
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
// ...
MIGRATOR.run(&pool).await.map_err(|error| StoreError::Migration {
    detail: error.to_string(),
})?;
```

已应用的版本记录在数据库里：

```text
$ sqlite3 /tmp/monitor-demo.db 'SELECT version, description, success FROM _sqlx_migrations;'
1|initial|1
```

（真实的表里还有 `installed_on`、`checksum`、`execution_time` 三列——校验和用来发现「已经跑过的迁移被人改过」，这是迁移系统的安全带。）

**把 schema 从 Rust 字符串里拿出来有三个收益：**

1. 数据库演进有了可审查的顺序（每个文件是一个版本，diff 就是变更）；
2. 不会在每个适配器方法里偷偷建表；
3. 迁移失败有专门的错误变体（`StoreError::Migration`），而不是混进普通查询错误。

**迁移的黄金规则：只增不改。** 已经发布的迁移文件一旦被应用过，就不能再修改——要改就加 `0002_xxx.sql`。校验和会让偷偷修改的行为在下次启动时暴露。

---

## 9. 深水区

### 9.1 SQLite 的并发模型：单写者

SQLite 允许**多个读者**或**一个写者**同时工作。写入是串行的，`max_connections(5)` 并不等于「5 个并发写」，它只是让读可以并行。

由此推出两条实践：

- **事务要短。** 长事务会占住写锁，其他写者只能等 `busy_timeout` 然后失败。
- **别在事务里做 I/O。** 网络请求、文件读写都不该出现在 `begin()` 和 `commit()` 之间。本项目正好符合：检查（网络）在事务之外，事务里只有两条 SQL。

### 9.2 WAL 让读不阻塞写

默认的回滚日志模式下，写者会阻塞读者。WAL 模式下写只追加到 `-wal` 文件，读者继续读主库的旧快照——**读与写可以同时进行**。代价是多出 `-wal` 与 `-shm` 两个文件，以及需要定期 checkpoint（SQLite 自动做）。

对一个健康监测服务来说这很关键：HTTP handler 在写检查结果的同时，另一个请求还能读列表。

### 9.3 「可空列组合」的补救：CHECK 约束

回到第 3 节那个阻抗失配问题。`check_results` 目前允许「`status` 与 `failure_kind` 同时为 NULL」这种无意义状态。数据库层面可以把它禁掉：

```sql
-- 0002_enforce_outcome_shape.sql（示例，尚未加入项目）
ALTER TABLE check_results
    ADD CONSTRAINT outcome_shape
    CHECK ((status IS NOT NULL) <> (failure_kind IS NOT NULL));
```

SQLite 对 `ALTER TABLE ... ADD CONSTRAINT` 的支持有限，实际做法通常是重建表并复制数据——**这正是迁移系统存在的理由**：难做的变更更需要版本化、可审查、可回滚。

### 9.4 为什么用自增 id 排序，而不是时间戳

| 依据 | 优点 | 风险 |
|---|---|---|
| `ORDER BY id DESC` | 严格单调，无并列歧义 | 只在单库单写者下成立 |
| `ORDER BY created_at DESC` | 跨库、跨进程都有意义 | 需要处理时钟回拨与同秒并列 |

本项目用前者，因为它只有一个数据库、一个写者。**换到多实例部署时，这个假设会失效**——那时该加时间戳并明确排序规则。

---

## 10. 常见误解

| 误解 | 准确说法 |
|---|---|
| 「内存库和文件库只差持久化」 | 内存库的每个连接是独立数据库，池子会产生「多个库」的错觉。 |
| 「连接池越大并发越高」 | SQLite 是单写者；池子只让读并行，写仍然排队。 |
| 「事务只是性能优化」 | 它保证「检查 + 写入」原子完成，去掉它会留下竞态窗口。 |
| 「`Database` 错误直接返回 500 就行」 | 还要保留 `source()`，否则排障时看不到 SQLite 的原始信息。 |
| 「迁移文件可以随时改」 | 已应用的迁移不能改；校验和会揭发它，正确做法是新增迁移。 |
| 「行数据不用再校验」 | 数据库可能存着旧版本或手工改过的数据，反序列化必须重新校验。 |

---

## 11. 练习与自测

### 练习

```sh
cargo test -p monitor-store
```

三项动手任务：

1. **观察内存库的陷阱**：把 `SqliteRepository::connect` 中对内存库的 `max_connections` 改成 2，运行测试，解释为什么会出现「表不存在」或「数据丢失」。
2. **写第二个迁移**：给 `check_results` 加上 `CHECK` 约束（第 9.3 节），并写一个测试断言「同时写入 `status` 与 `failure_kind` 会被拒绝」。
3. **验证索引**：用 `EXPLAIN QUERY PLAN` 查看 `latest_result` 的查询计划，确认它走了 `check_results_target_id_id` 索引。

### 自测清单（能全部做到才算掌握）

1. 说出异步 trait 方法三种写法各自的一个代价，并解释本项目为什么放弃 `dyn`。
2. 解释为什么内存库的池子必须是一个连接，用「每个连接一个数据库」说明。
3. 说明 `check_results` 如何用可空列编码枚举，以及这种方式无法禁止哪种非法状态。
4. 说出写路径里的两层保护（事务与领域校验）各自防什么。
5. 解释 `RETURNING` 比「`INSERT` 后查 `last_insert_rowid()`」好在哪。
6. 说明为什么必须保留 `Error::source()`，它和枚举变体各自解决什么。
7. 说出迁移的黄金规则与校验和的作用。
8. 解释 WAL 让「读不阻塞写」的原理，以及多出来的两个文件是什么。

---

## AI 辅导提示词

这三段可以直接复制给 AI 助手（Kimi、ChatGPT 等）。它们的设计意图是**让助手出题和追问，而不是替你写代码**——完整方法论见[用 AI 助手当教练](../guided/ai-tutor.md)。

```text
下面是我的表结构和写入事务。请回答三个问题：
1. 这个 schema 允许哪些无意义的状态组合？
2. 我的事务是否过长、是否包含 I/O？
3. 有没有查询缺少索引，或者排序依据在并发下会歧义？
只给分析，不要改代码。

[粘贴 schema 与写入函数]
```

```text
请出 3 道判断题，考察「领域错误与数据库错误的边界」：
给定几种失败（找不到目标、schema 校验失败、磁盘满、行数据被手工改坏），
让我判断该映射成哪个 StoreError 变体、哪个 HTTP 状态码、是否该重试。
我答完后逐条点评。
```

```text
我要给这个表加一个约束。请先问我三个问题
（现有数据是否满足约束、SQLite 是否支持在线变更、失败时如何回滚），
再让我自己写出迁移文件的步骤。不要直接给 SQL。
```
