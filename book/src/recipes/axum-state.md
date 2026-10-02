# Axum State：共享状态放在哪里

| 元数据 | 内容 |
|---|---|
| 任务 | 把仓储与检查器交给 handler，同时保持可测试 |
| 概念 | `State`、`with_state`、Router 的类型参数、`Arc` |
| 错误码 | `E0308`（状态类型不匹配） |
| 先修 | 泛型、`Arc`、handler 的职责边界 |
| 项目阶段 | Web 层 `app` 与 `AppState` |

## 现象：忘记 `with_state` 时，错误出现在别处

```rust,ignore
async fn handler(State(state): State<AppState>) -> String {
    state.name.clone()
}

fn main() {
    // 忘记 .with_state(...)
    let _router: Router = Router::new().route("/", get(handler));
}
```

```text
error[E0308]: mismatched types
   |
16 |     let _router: Router = Router::new().route("/", get(handler));
   |                                         -----      ^^^^^^^^^^^^
   |                                         |          expected `MethodRouter`,
   |                                         |          found `MethodRouter<AppState>`
   |
   = note: expected struct `MethodRouter<()>`
              found struct `MethodRouter<AppState>`
```

这个报错很有教育意义：**状态类型是 Router 类型的一部分**。加了 `#[derive(Clone)]` 的 `AppState` 不会凭空消失，它会一直留在 `MethodRouter<AppState>` 里，直到你调用 `with_state` 把它变成 `Router<()>`。

两个推论：

1. 忘记 `with_state` 时，**错误不一定出现在那一行**——它出现在你要求「一个状态为 `()` 的 Router」的地方（变量标注、`axum::serve` 调用）；
2. 报错里的 `MethodRouter<AppState>` 就是「还差一个 `with_state`」的信号。

## 判断：三处可放状态，选哪个

| 放法 | 适用 | 代价 |
|---|---|---|
| `with_state` + `State<T>`（本项目） | 应用级共享依赖（仓储、检查器、配置） | 每个 handler 的签名多一个参数 |
| `Extension<T>` | 中间件动态插入的值、每请求不同的东西 | 取不到时是运行期 500，不是编译错误 |
| 全局 `static` / `OnceLock` | 真正的进程级常量 | 测试无法替换，破坏可测试性 |

**默认用 `with_state`**：它在编译期保证「handler 需要的东西一定被提供了」。

## 修复选择

```rust,ignore
pub fn app<R>(repository: Arc<R>, checker: HealthChecker) -> Router
where
    R: MonitorRepository + 'static,
{
    Router::new()
        .route("/targets", get(list_targets::<R>).post(create_target::<R>))
        .with_state(AppState { repository, checker })     // ← 类型在这里被擦除
}
```

两个容易忽略的细节：

- **泛型参数要传给 handler**：`get(list_targets::<R>)` 里的 turbofish 告诉编译器用哪个 `R`，否则推断不出来。
- **`AppState` 的 `Clone` 应手写**：`#[derive(Clone)]` 会加上 `R: Clone` 约束，而仓储不需要可克隆——共享已经由 `Arc` 完成了。

## 项目里的位置

```rust,ignore
struct AppState<R> {
    repository: Arc<R>,
    checker: HealthChecker,      // 内部持有 reqwest::Client，本身已是 Arc 语义
}
```

测试传 `Arc::new(InMemoryRepository::new())`，生产传 `Arc::new(SqliteRepository::connect(...).await?)`，handler 一个字不用改。

深入阅读：[用 Axum 暴露 HTTP 接口](../web/axum-api.md#3-state共享状态放在哪里)。
