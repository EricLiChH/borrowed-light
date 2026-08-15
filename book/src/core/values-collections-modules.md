# 值、枚举、集合与模块边界

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 把原始输入变成可遍历的领域数据 | expression、struct、enum、match、slice、Vec、HashMap、module | 150 分钟 | 监测目标与结果汇总 |

Rust 的 `if`、`match` 和代码块都是表达式（expression）：最后一个不带分号的值会成为结果。把状态建模为 enum，通常比布尔值加若干可空字段更难写错。

```rust
#[derive(Debug)]
enum Outcome {
    Reachable(u16),
    Unreachable(String),
}

fn label(outcome: &Outcome) -> &'static str {
    match outcome {
        Outcome::Reachable(200..=299) => "healthy",
        Outcome::Reachable(_) => "responded",
        Outcome::Unreachable(_) => "unreachable",
    }
}

fn main() {
    assert_eq!(label(&Outcome::Reachable(204)), "healthy");
}
```

## `Copy`、move 与 drop

`u16` 实现 `Copy`，赋值后两边都能继续用；`String` 管理堆内存，默认会 move，离开最后一个所有者的作用域时执行 drop。不要把“在栈上/堆上”当成唯一判断标准；先看类型是否实现 `Copy`，再看接口是否接管 `T`。

```rust,compile_fail
fn main() {
    let name = String::from("Rust");
    let moved = name;
    println!("{name} -> {moved}");
}
```

## slice 是集合的借用窗口

```rust
fn healthy_count(statuses: &[u16]) -> usize {
    statuses
        .iter()
        .filter(|&&status| (200..300).contains(&status))
        .count()
}

fn main() {
    let statuses = vec![200, 204, 404, 503];
    assert_eq!(healthy_count(&statuses[0..3]), 2);
}
```

`&[u16]` 不关心调用者使用数组还是 Vec，也不接管集合。迭代器先借用，再由 `filter`、`map`、`collect` 组合转换；普通循环更清楚时就用循环。

## 模块把不变量藏在入口后面

```rust
mod monitor {
    pub struct Target { name: String }

    impl Target {
        pub fn new(name: &str) -> Option<Self> {
            (!name.trim().is_empty()).then(|| Self { name: name.into() })
        }

        pub fn name(&self) -> &str { &self.name }
    }
}

fn main() {
    let target = monitor::Target::new("Rust").unwrap();
    assert_eq!(target.name(), "Rust");
}
```

字段保持私有，调用者只能经过构造函数。依次完成 `10_collections` 与 `11_modules`，再在项目中找出哪些字段不应公开。
