# 进阶练习的分层提示

Rustlings 的 `h` 只显示第一层方向。仍卡住时，才在这里逐层展开；第三层包含代码形状、取舍和参考解入口。

## 04_result

<details><summary>第二层：关键 API</summary>

用 `parse::<u16>()`、`map_err` 和 `?` 处理格式错误；范围规则仍用显式条件判断。

</details>
<details><summary>第三层：形状与取舍</summary>

先 `let status = raw.parse::<u16>().map_err(...)?;`，再判断合法范围。`?` 传播异常路径，业务范围仍清楚留在正文。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/errors/04_result.rs)

</details>

## 05_repository

<details><summary>第二层：关键 API</summary>

为 `Vec<Target>` 实现 `TargetSource`，使用 `slice::last` 返回 `Option<&Target>`。

</details>
<details><summary>第三层：形状与取舍</summary>

返回引用让数据源保留所有权；clone 虽容易通过，却掩盖接缝的借用契约。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/traits/05_repository.rs)

</details>

## 06_summary

<details><summary>第二层：关键 API</summary>

依次使用 `iter`、`filter`、`count`；`filter` 的闭包参数会多一层引用。

</details>
<details><summary>第三层：形状与取舍</summary>

形状是 `statuses.iter().filter(|&&status| ...).count()`。若解引用让意图更模糊，普通循环同样惯用。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/iterators/06_summary.rs)

</details>

## 07_lifetimes

<details><summary>第二层：关键 API</summary>

给两个输入和返回值使用同一个命名生命周期 `'a`。

</details>
<details><summary>第三层：形状与取舍</summary>

`fn longer<'a>(left: &'a str, right: &'a str) -> &'a str` 描述共同约束；它不会延长任一输入。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/lifetimes/07_lifetimes.rs)

</details>

## 08_borrowed_struct

<details><summary>第二层：关键 API</summary>

使用 `slice::first` 与 `Option::map`，把元素引用放进 `Selection`。

</details>
<details><summary>第三层：形状与取舍</summary>

`names.first().map(|name| Selection { name })` 不复制 String；代价是 Selection 不能比 names 活得更久。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/lifetimes/08_borrowed_struct.rs)

</details>

## 09_borrowed_slice

<details><summary>第二层：关键 API</summary>

直接在输入 `&str` 上使用 `trim_start_matches`、`split`、`next`。

</details>
<details><summary>第三层：形状与取舍</summary>

链式调用返回输入内部切片；先建局部 String 再借用会形成悬垂引用。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/lifetimes/09_borrowed_slice.rs)

</details>

## 10_collections

<details><summary>第二层：关键 API</summary>

使用 `HashMap::entry(label).or_insert(0)` 取得计数器。

</details>
<details><summary>第三层：形状与取舍</summary>

判断 `(200..300).contains(status)` 后执行 `*entry += 1`。两个固定桶也可用 struct；HashMap 适合标签将扩展的情况。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/collections/10_collections.rs)

</details>

## 11_modules

<details><summary>第二层：关键 API</summary>

实现 `pub fn new(name: &str) -> Option<Self>` 与 `pub fn name(&self) -> &str`。

</details>
<details><summary>第三层：形状与取舍</summary>

字段保持私有，构造器用 `then` 维护非空规则。公开字段更短，但会让不变量散落到调用者。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/modules/11_modules.rs)

</details>

## 12_threads

<details><summary>第二层：关键 API</summary>

使用 `mpsc::channel`、`Sender::clone`、`thread::spawn`；所有 sender drop 后再收集。

</details>
<details><summary>第三层：形状与取舍</summary>

每个 move 闭包拥有一个 sender，主线程通过 `receiver.iter().collect()` 汇总并排序。消息传递避免共享 Vec 的锁竞争。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/concurrency/12_threads.rs)

</details>

## 13_future

<details><summary>第二层：关键 API</summary>

立即就绪的 async 函数可以直接把 `status` 作为尾表达式返回。

</details>
<details><summary>第三层：形状与取舍</summary>

函数体写 `status` 后，第一次 poll 得到 `Ready(status)`。该练习只建立惰性/poll 基线；Pending 在练习 16 验证。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/async/13_future.rs)

</details>

## 14_send_sync

<details><summary>第二层：关键 API</summary>

外层用 `Arc` 共享所有权，内层用 `Mutex` 同步 Vec 的修改。

</details>
<details><summary>第三层：形状与取舍</summary>

构造 `Arc::new(Mutex::new(Vec::new()))`。`Rc<RefCell<_>>` 适合单线程，不满足这里的 `Send + Sync`。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/concurrency/14_send_sync.rs)

</details>

## 15_retry_policy

<details><summary>第二层：关键 API</summary>

用 `attempt < max_attempts` 和 `matches!` 组合两个必要条件。

</details>
<details><summary>第三层：形状与取舍</summary>

只匹配 `Connect | Timeout`；redirect/builder 立即结束。扩大重试范围可能放大确定性失败和副作用。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/async/15_retry_policy.rs)

</details>

## 16_backpressure_cancel

<details><summary>第二层：关键 API</summary>

在 `tokio::select!` 中让立即 Ready 的 shutdown 与 `&mut blocked_send` 竞争；用 `biased;` 固定 Ready 分支优先级。

</details>
<details><summary>第三层：形状与取舍</summary>

shutdown 分支返回 `true`，send 分支返回 `false`；select 后 drop send future，并断言 receiver 只有消息 1。真实服务不应滥用 `biased`，这里仅用于确定性教学。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/async/16_backpressure_cancel.rs)

</details>

## 17_web_state

<details><summary>第二层：关键 API</summary>

把 `RwLock<Vec<String>>` 放进 `Arc`，再把 Arc 放进可 Clone 的 AppState。

</details>
<details><summary>第三层：形状与取舍</summary>

`AppState { targets: Arc::new(RwLock::new(Vec::new())) }` 让 clone 共享同一列表；生产 Axum 使用 Tokio 锁或消息设计，练习只验证所有权形状。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/web/17_web_state.rs)

</details>

## 18_storage_mapping

<details><summary>第二层：关键 API</summary>

对 `Outcome` 使用 `match`；成功填 status，失败填 kind/reason。

</details>
<details><summary>第三层：形状与取舍</summary>

两个分支直接返回互斥 tuple。可空数据库列是适配器表示，领域层继续使用 enum 防止矛盾组合。[参考解](https://github.com/EricLiChH/borrowed-light/blob/main/exercises/solutions/storage/18_storage_mapping.rs)

</details>
