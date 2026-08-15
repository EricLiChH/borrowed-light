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

```sh
cd exercises
rustlings run 05_repository
```

先让练习失败，再按“方向 → API → 形状”三层提示修复。完成标准：能在不 clone `Target` 的情况下替换数据来源，并解释对象安全（object safety）为什么会影响 `dyn Trait` 的方法形状。
