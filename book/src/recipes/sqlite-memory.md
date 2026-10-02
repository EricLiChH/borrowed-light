# SQLite 内存测试：为什么数据突然消失

| 元数据 | 内容 |
|---|---|
| 任务 | 让「临时数据库」的测试既快又正确 |
| 概念 | SQLite 连接模型、连接池、WAL |
| 错误码 | 无（运行期报 `no such table`） |
| 先修 | 连接池概念、`SqliteRepository::connect` |
| 项目阶段 | 存储层测试 |

## 现象

测试里明明建了表，下一个查询却报：

```text
Error: in prepare, no such table: t
```

用命令行直接验证这条规则（**每次 `sqlite3 ":memory:"` 都是一次新连接，也就是一个全新的空数据库**）：

```text
$ sqlite3 ':memory:' "CREATE TABLE t(x); INSERT INTO t VALUES (1); SELECT '建表并写入成功';"
建表并写入成功

$ sqlite3 ':memory:' 'SELECT * FROM t;'
Error: in prepare, no such table: t
```

## 判断

**SQLite 的每个纯内存连接都是一个独立数据库。** 这不是 bug，是 `:memory:` 的定义：数据库活在连接的内存里，连接结束就消失。

由此推出：**连接池开到两个，就等于同时存在两个互不相干的空数据库。** 你的迁移跑在连接 A 上，查询却拿到连接 B——于是「刚建的表突然不存在」。

## 修复选择

| 选择 | 做法 | 适合 |
|---|---|---|
| 内存库固定单连接（本项目） | 池子 `max_connections(1)` | 快速契约测试；简单可靠 |
| 共享缓存内存库 | `file:memdb1?mode=memory&cache=shared` | 需要多连接的内存库 |
| 用临时文件 | 每个测试一个独立文件，析构时删除 | 想验证真实的文件路径代码 |
| 用 `tempfile` 之类 | 由库保证清理 | 不想手写清理逻辑 |

本项目的做法是把规则**写进适配器**，而不是让每个调用方记住：

```rust,ignore
let in_memory = options.get_filename().to_string_lossy() == ":memory:";
let max_connections = if in_memory { 1 } else { FILE_POOL_SIZE };
```

## 相关的另一半：文件库要用连接池

内存库限制为 1 个连接，但**文件库不该这么做**——那会把所有查询串行化。文件库的正确配置是连接池 + WAL：

```rust,ignore
let options = if in_memory {
    options
} else {
    options.journal_mode(SqliteJournalMode::Wal)     // 读不阻塞写
};
```

## 测试怎么写

```rust,ignore
// 内存库：快，用于契约测试
let repository = SqliteRepository::connect("sqlite::memory:").await?;

// 文件库：临时文件 + 析构清理 -wal/-shm
struct TempDatabase { path: PathBuf }
impl Drop for TempDatabase {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", self.path.display()));
        }
    }
}
```

**别忘了 `-wal` 与 `-shm`**：它们和主文件一样会被留在磁盘上。

深入阅读：[从内存切换到 SQLite + SQLx](../web/sqlite-storage.md#7-连接池的两种形状)。
