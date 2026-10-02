# 用 trait 建立第一个可替换接缝

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 让内存目标列表共用读取方式 | trait、泛型、借用、集成测试 | 120 分钟 | 同步 `TargetSource` 接口 |

这里暂时不出现 `async`、`Send + Sync` 或数据库。先用一个同步 trait 学会：调用者只依赖能力，具体集合隐藏在实现之后。

```rust
#[derive(Debug)]
struct Target { name: String }

trait TargetSource {
    fn latest(&self) -> Option<&Target>;
}

impl TargetSource for Vec<Target> {
    fn latest(&self) -> Option<&Target> {
        self.last()
    }
}

fn latest_name(source: &impl TargetSource) -> Option<&str> {
    source.latest().map(|target| target.name.as_str())
}

fn main() {
    let targets = vec![Target { name: "Rust".into() }];
    assert_eq!(latest_name(&targets), Some("Rust"));
}
```

先预测：为什么返回 `Option<&Target>` 而不是 clone 一份？`impl TargetSource` 与 `&dyn TargetSource` 各自把什么决定留给编译期或运行期？

## 接缝是否值得存在

一个 trait 应该隐藏真实复杂度，而不是把每个结构体都包装一层。本阶段的接口很小，目的是练习借用与替换；到 Web 阶段，`MonitorRepository` 才会升级为异步接口，并真正隐藏 `RwLock`、migration、SQL 和行映射。

## 对象安全：一个会一直跟到 Web 阶段的决定

`dyn Trait` 只接受“对象安全”的方法：不能有泛型参数，不能返回 `Self`，也不能是 `async fn`。这解释了本阶段练习里那个看起来别扭的签名——返回 `Option<&Target>` 而不是 `Target`，恰好也是为了让 `dyn` 可用。

到异步存储阶段，这个限制会真正咬人：`async fn` 既不是对象安全的，它的 future 也不满足 `Send`，而 axum 的 handler 必须 `Send`。届时有两种出路——把 future 装箱（`#[async_trait]` 做的就是这件事，每次调用多一次分配），或者放弃 `dyn`、改用泛型参数。本项目选择后者，见 `docs/adr/0017`。

所以这一节的完成标准里，对象安全不是背景知识，而是你以后每次写 `dyn` 都要先算的一笔账。

```sh
cd exercises
rustlings run 05_repository
```

先让练习失败，再按“方向 → API → 形状”三层提示修复。完成标准：能在不 clone `Target` 的情况下替换数据来源，并解释对象安全为什么会影响 `dyn Trait` 的方法形状。
