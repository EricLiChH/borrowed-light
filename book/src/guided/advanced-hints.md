# 进阶练习的三层提示

先只展开方向；仍卡住时再看 API；最后才看代码形状。Rustlings 的 `h` 会显示同样三层摘要。

| 练习 | 方向 | 关键 API | 最后形状 |
|---|---|---|---|
| `07_lifetimes` | 返回引用来自输入 | 命名生命周期 | 两个输入和返回值共用 `'a` |
| `08_borrowed_struct` | 结构体字段借用集合元素 | `first`、`map` | `Selection { name }` 不 clone |
| `09_borrowed_slice` | 返回输入内部的一段 | `split`、`next` | 在 `&str` 上链接调用 |
| `10_collections` | 遍历并累计两个桶 | `HashMap::entry` | `*entry.or_insert(0) += 1` |
| `11_modules` | 私有字段，公开能力 | `pub fn`、`Option<Self>` | 构造器校验，getter 借用 |
| `12_threads` | worker 移动 sender | `mpsc`、`thread::spawn` | 关闭 sender 后收集并排序 |
| `13_future` | future 被 poll 才执行 | `Future::poll` | async 函数直接返回 status |
| `14_send_sync` | 所有权与同步缺一不可 | `Arc<Mutex<T>>` | clone Arc，不 clone Vec |
| `15_retry_policy` | 类别和次数共同决定 | `matches!` | 只允许 Connect/Timeout |
| `16_web_state` | handler clone 共享同一状态 | `Arc<RwLock<T>>` | 锁在 Arc 内部 |
| `17_storage_mapping` | enum 分支映射可空列 | `match` | 成功与失败列互斥 |

参考解位于 `exercises/solutions/<目录>/<编号>.rs`。打开前先写下自己的失败原因；只比较“为什么接口不同”，不要逐字符抄答案。
