# 对象安全：为什么 `dyn` 拒绝 `async fn`

| 元数据 | 内容 |
|---|---|
| 任务 | 决定一个异步 trait 该装箱、该泛型化，还是根本不该存在 |
| 概念 | object safety、RPITIT、`Send` bound、装箱的代价 |
| 错误码 | `E0038`、`E0277` |
| 先修 | trait 基础、`Send`/`Sync` |
| 项目阶段 | 存储接缝 `MonitorRepository` |

把 `async fn` 直接写进一个要当 `dyn` 用的 trait，会同时挨两个错：

```text
error[E0038]: the trait `MonitorRepository` is not dyn compatible
error[E0277]: `impl Future<Output = ...>` cannot be sent between threads safely
```

第二个错误和你写没写 `Send` 无关。trait 里的 `async fn` 返回一个不透明 future，编译器不保证它 `Send`，而 axum 的 handler 必须 `Send`。

## 判断

问一句：**运行期真的需要在同一个变量里放不同的实现吗？**

- 需要（插件注册表、`Vec<Box<dyn T>>`）：只能装箱。
- 不需要（本项目只有一个适配器被注入）：泛型更便宜，也更清楚。

## 三种写法

| 写法 | 装箱 | 能用于 `dyn` | axum handler 可用 |
|---|---|---|---|
| `async fn` in trait | 否 | 否（`E0038`） | 否（future 不保证 `Send`） |
| `#[async_trait]` | 每次调用一次 | 是 | 是 |
| `-> impl Future + Send` | 否 | 否 | 是 |

第三种是本项目选的：实现方照旧写 `async fn`，调用方拿到不装箱的 future，代价是放弃 `dyn`——于是 `app` 改成对 `R: MonitorRepository` 泛型，`with_state` 再把 `R` 擦除，调用方看到的仍然是一个具体的 `Router`。

## 修复选择

1. 先确认这个 trait 是否真的需要运行期多态。多数“可替换接缝”只需要编译期替换，尤其是只有一个生产实现的时候。
2. 确实需要 `dyn`，就接受装箱，并把它写进文档，而不是让宏悄悄替你决定。
3. 只要 `Send` 不要 `dyn`，用第三种写法；如果连 `Send` 都不需要，才轮到裸 `async fn`。

决策全文见仓库的 `docs/adr/0017`。
