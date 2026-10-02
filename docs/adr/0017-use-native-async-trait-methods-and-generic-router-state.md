# 用原生 async trait 方法，并让路由器状态泛型化

存储接缝最初写成 `#[async_trait]`。它可用，但每次调用都会把 future 装箱，每个查询多一次分配和一次堆间接；而且课程要教的第一课是“看到 `dyn` 就要问代价”，宏把这个代价藏了起来。

Rust 现在允许在 trait 里用 `-> impl Future<Output = ...> + Send` 声明异步方法（RPITIT）。实现方仍然写普通的 `async fn`，编译器不做装箱。代价是这类方法不能用于 `dyn`，因此 `monitor_web::app` 从 `Arc<dyn MonitorRepository>` 改成对 `R: MonitorRepository` 泛型，`AppState` 通过手写 `Clone` 避免误加 `R: Clone` 约束。`with_state` 会擦除 `R`，所以 `app` 的返回类型仍然是具体的 `Router`。

`async fn` 直接写在 trait 里仍然不可取：它的 future 不被认为满足 `Send`，而 axum handler 必须 `Send`。三种写法的取舍记录在 `monitor-store/src/lib.rs` 的 crate 文档里。

当某个接缝确实需要在运行时持有异构的实现集合时，才回到装箱方案；本项目的适配器在编译期就确定，不需要。
