# Rust 语法速查

> **用法**：这不是教程，是**查阅页**。读章节时遇到记不清的语法，回到这里看一眼就走。
>
> 每条都尽量给出最小例子与"什么时候用它"。带 ✅ 的例子由 `mdbook test` 编译验证。

---

## 1. 变量与绑定

| 语法 | 例子 | 说明 |
|---|---|---|
| 不可变绑定 | `let name = String::from("Rust");` | 默认不可变，这是 Rust 的默认值 |
| 可变绑定 | `let mut count = 0;` | 需要修改才加 `mut` |
| 遮蔽 | `let name = name.len();` | 产生新绑定，**可以换类型** |
| 类型标注 | `let port: u16 = 8080;` | 推断不出来时才写 |
| 常量 | `const TIMEOUT_MS: u64 = 5_000;` | 编译期内联，每个使用点各有一份 |
| 静态量 | `static NAME: &str = "monitor";` | 全局唯一，有固定地址 |
| 丢弃值 | `let _ = result;` | **立刻**丢弃（`Drop` 立即运行） |
| 保留守卫 | `let _guard = lock();` | 值活到作用域结束 |
| 提前退出 | `let Some(x) = opt else { return; };` | let-else：不匹配就离开作用域，绑定之后可用 |

```rust,editable
fn main() {
    let url = String::from("https://example.com");
    let url = url.len();              // 遮蔽：类型从 String 变成 usize
    let mut hits = 0;                 // 可变
    hits += 1;

    const PORT: u16 = 8080;           // 常量
    let port: u16 = PORT;

    println!("{url} {hits} {port}");
}
```

---

## 2. 类型与转换

| 语法 | 例子 | 说明 |
|---|---|---|
| 整数类型 | `u8 u16 u32 u64 usize` / `i8 … isize` | 下标与长度用 `usize` |
| 浮点 | `f32 f64` | 默认 `f64` |
| 文本 | `String`（拥有）/ `&str`（借用） | 参数优先用 `&str` |
| 切片引用 | `&[T]` | 参数优先用 `&[T]` 而不是 `&Vec<T>` |
| 元组 | `let (a, b) = (1, "x");` | 固定长度、可异构 |
| 数组 | `let xs: [u8; 3] = [1, 2, 3];` | 长度是类型的一部分 |
| `Option` | `Option<T>` | 「没有值」是正常情况 |
| `Result` | `Result<T, E>` | 「可能失败」 |
| 非零整数 | `NonZeroUsize` | 让零值无法表示 |
| 数值转换 | `u16::try_from(n)?` | 可能失败用 `try_from` |
| 数值转换 | `u64::from(x)` | 保证不失败时用 `from` |
| 字符串解析 | `"8080".parse::<u16>()?` | 返回 `Result` |
| 智能指针 | `Box<T>` / `Rc<T>` / `Arc<T>` | 堆分配 / 单线程共享 / 跨线程共享 |
| 内部可变性 | `Cell<T>` / `RefCell<T>` / `Mutex<T>` | 单线程 / 运行期借用检查 / 跨线程 |

```rust,editable
fn main() {
    let wide: u64 = 8080;
    let narrow: u16 = u16::try_from(wide).expect("8080 放得进 u16");

    let text = String::from("443");
    let port: u16 = text.parse().expect("应该是端口号");

    println!("{narrow} {port} {:?}", narrow.checked_add(1));
}
```

**规则**：`From` 用于保证不失败的转换，`TryFrom` 用于可能失败的转换。看到 `as` 就该警惕——它会静默截断。

---

## 3. 结构体、枚举与模式

| 语法 | 例子 |
|---|---|
| 具名字段 | `struct Target { name: String, url: Url }` |
| 元组结构体 | `struct Names(Vec<String>);` |
| 单元结构体 | `struct Marker;` |
| 派生常用 trait | `#[derive(Debug, Clone, PartialEq, Eq, Hash)]` |
| 枚举（可带数据） | `enum Outcome { Reachable { status: u16 }, Unreachable(String) }` |
| 方法 | `impl Target { fn name(&self) -> &str { &self.name } }` |
| 关联函数 | `impl Target { fn new(...) -> Result<Self, Error> }` |
| 结构体更新 | `Target { enabled: false, ..base }` |
| 字段简写 | `Target { name, url }` |

模式（`match` / `if let` / `let else` 通用）：

| 模式 | 例子 | 匹配什么 |
|---|---|---|
| 字面量 | `200` | 精确值 |
| 范围 | `200..=299` | 闭区间 |
| 或 | `400 \| 404` | 任一 |
| 绑定 | `status` | 任意值并绑定 |
| 绑定 + 范围 | `code @ 200..=299` | 两者都要 |
| 结构体解构 | `Outcome::Reachable { status }` | 按字段名 |
| 元组/切片 | `(a, b)` / `[first, rest @ ..]` | 位置解构 |
| 忽略 | `..` / `_` | 明确不关心 |
| 守卫 | `n if n > 0` | 模式之外的额外条件 |

```rust,editable
#[derive(Debug)]
enum Outcome {
    Reachable { status: u16 },
    Unreachable(String),
}

fn label(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Reachable { status: 200 } => String::from("ok"),
        Outcome::Reachable { status } if *status < 300 => format!("success {status}"),
        Outcome::Reachable { status } => format!("other {status}"),
        Outcome::Unreachable(reason) => format!("down: {reason}"),
    }
}

fn main() {
    println!("{}", label(&Outcome::Reachable { status: 200 }));
    println!("{}", label(&Outcome::Unreachable(String::from("timeout"))));
}
```

**穷尽性由编译器强制**：漏掉一个变体是编译错误（`E0004`），不是运行期默认分支。
---

## 4. 函数、泛型与 trait

| 语法 | 例子 | 说明 |
|---|---|---|
| 函数 | `fn check(target: &MonitorTarget) -> CheckResult` | 最后一行不加分号就是返回值 |
| 提前返回 | `return CheckResult::reachable(target, 200);` | 只在提前退出时用 |
| 泛型函数 | `fn newest<T, F>(items: &[T], key: F) -> Option<&T>` | 编译期展开（单态化） |
| 内联约束 | `fn log<T: Display>(value: T)` | 约束少时可读性好 |
| where 子句 | `where T: Display, F: Fn(&T) -> String` | 约束多时更清楚 |
| impl Trait 参数 | `fn log(value: impl Display)` | 只有一个参数、不需要命名时 |
| impl Trait 返回 | `fn numbers() -> impl Iterator<Item = u16>` | 隐藏具体类型，不装箱 |
| 关联类型 | `trait Source { type Item; }` | 一个类型只有一种实现方式时 |
| 泛型参数 | `trait From<T>` | 一个类型有多种实现方式时 |
| 默认方法 | `fn has_any(&self) -> bool { self.latest().is_some() }` | 实现方可以不写 |
| trait 约束下的方法调用 | `fn f<T: Display>(v: T) { println!("{v}"); }` | 需要 trait 在作用域内 |
| dyn trait | `Box<dyn Error>` / `&dyn Display` | 运行期分派，需要对象安全 |
| 派生 | `#[derive(Debug, Clone)]` | 编译器生成实现 |

关联类型与泛型参数的判断标准：**「一个类型能有几种实现方式」**——一种用关联类型（`Iterator::Item`），多种用泛型参数（`From<T>`）。

---

## 5. 所有权与借用速记

| 规则 | 一句话 |
|---|---|
| 唯一所有者 | 每个值在任一时刻只有一个所有者 |
| 移动 | `let b = a;` 之后 `a` 不可用（除非 `Copy`） |
| 借用 | `&T` 只读、`&mut T` 独占读写 |
| 借用范围 | 从创建到最后一次使用（NLL），不是花括号 |
| 冲突判据 | 同一时间只能有「多个 `&T`」或「一个 `&mut T`」 |
| `Copy` | 逐位复制即正确的类型（`u16`、`bool`、`&T`） |
| `Clone` | 显式复制，可能有堆分配 |
| `Drop` | 所有者离开作用域时运行；实现 `Drop` 的类型不能部分移动 |

| 参数形状 | 函数获得 | 调用后原绑定 |
|---|---|---|
| `T` | 所有权 | 通常不可再用 |
| `&T` | 只读权限 | 可继续使用 |
| `&mut T` | 独占读写 | 借用结束后可用 |
| `impl Into<String>` | 任意可转换类型 | 取决于实参 |

```rust,editable
fn describe(target: &str) -> usize {   // 借：调用方保留所有权
    target.len()
}

fn consume(target: String) -> usize {  // 交：调用方失去所有权
    target.len()
}

fn main() {
    let owned = String::from("https://example.com");
    println!("{}", describe(&owned));   // owned 仍然可用
    println!("{}", consume(owned));     // owned 在这里被移动
    // println!("{owned}");             // 取消注释会看到 E0382
}
```

---

## 6. 闭包

| 语法 | 例子 | 说明 |
|---|---|---|
| 基本形式 | `\|status\| status < 300` | 参数类型通常可推断 |
| 多语句 | `\|x\| { let y = x + 1; y * 2 }` | 需要花括号 |
| 移动捕获 | `move \|\| format!("{name}")` | 把捕获的值移进闭包 |
| 作为参数 | `F: Fn(&T) -> String` | 只读、可多次调用 |
| 可变状态 | `F: FnMut() -> u32` | 会修改捕获的环境 |
| 只调用一次 | `F: FnOnce() -> T` | 消耗捕获的值 |
| 返回闭包 | `fn make() -> impl Fn(u16) -> bool` | 通常需要 `move` + `'static` |

包含关系：**`Fn` ⊂ `FnMut` ⊂ `FnOnce`**。参数位置能宽就宽（接收方要求越少，能传进来的越多）。

```rust,editable
fn make_checker(threshold: u16) -> impl Fn(u16) -> bool {
    move |status| status < threshold      // move：threshold 是函数参数，必须带走
}

fn main() {
    let healthy = make_checker(300);
    println!("{} {}", healthy(204), healthy(500));

    let mut count = 0;
    let mut tally = || { count += 1; count };   // FnMut：修改捕获的环境
    println!("{} {}", tally(), tally());
}
```

---

## 7. 错误处理

| 语法 | 例子 | 说明 |
|---|---|---|
| 传播 | `let port = text.parse::<u16>()?;` | 提前返回 + `From::from` 转换 |
| 在 Option 上 | `let first = text.chars().next()?;` | 只能在返回 `Option` 的函数里用 |
| Option → Result | `value.ok_or(TargetError::InvalidUrl)?` | 把「没有」升级成「错误」 |
| 惰性构造错误 | `value.ok_or_else(\|\| Error::from(raw.clone()))?` | 避免无条件构造 |
| 包装底层错误 | `text.parse().map_err(\|source\| Error::BadPort { source })` | 保留原因 |
| 默认值 | `unwrap_or(0)` / `unwrap_or_else(\|\| compute())` | 后者只在需要时求值 |
| 转换失败即失败 | `Result<Vec<u16>, _> = iter.map(..).collect()` | 收集遇到第一个错误就返回 |
| 期望 panic | `.expect("为什么我认为这里安全")` | 比 `unwrap()` 多了理由 |
| 错误链 | `fn source(&self) -> Option<&(dyn Error + 'static)>` | 保留因果链 |

```rust,editable
use std::num::ParseIntError;

fn port(text: &str) -> Result<u16, String> {
    text.parse::<u16>()
        .map_err(|error: ParseIntError| format!("invalid port {text:?}: {error}"))
}

fn main() {
    println!("{:?}", port("8080"));
    println!("{:?}", port("http"));

    let ports: Result<Vec<u16>, _> = ["80", "443"].iter().map(|t| t.parse::<u16>()).collect();
    println!("{ports:?}");
}
```

---

## 8. 集合与迭代器

| 语法 | 例子 | 说明 |
|---|---|---|
| 向量 | `vec![1, 2, 3]` / `Vec::with_capacity(8)` | 预知数量时先分配 |
| 安全索引 | `values.get(0)` | 越界返回 `None`，不 panic |
| 三种遍历 | `&vec` / `&mut vec` / `vec` | 产出 `&T` / `&mut T` / `T` |
| 原地过滤 | `values.retain(\|v\| *v < 500)` | 无额外分配 |
| 排序 | `values.sort_by_key(\|v\| *v)` | 稳定排序 |
| 映射 | `iter.map(\|x\| x * 2).collect::<Vec<_>>()` | 惰性 |
| 过滤 | `iter.filter(\|x\| **x > 0)` | 惰性 |
| 折叠 | `iter.fold(0, \|acc, x\| acc + x)` | 累加 |
| 查找 | `iter.find(\|x\| **x == 404)` | 返回 `Option<&T>` |
| `HashMap` 插入或更新 | `map.entry(key).or_insert(0)` | 一次哈希查找 |
| `HashMap` 仅存在时修改 | `.and_modify(\|v\| *v += 1).or_insert(0)` | 链式 |
| 字符串拼接 | `format!("{a}{b}")` / `s.push_str(b)` | 前者一次分配 |
| 字符遍历 | `text.chars()` / `text.char_indices()` | `len()` 是字节数 |

```rust,editable
use std::collections::HashMap;

fn main() {
    let mut counts: HashMap<u16, usize> = HashMap::new();
    for status in [200_u16, 200, 404] {
        *counts.entry(status).or_insert(0) += 1;
    }

    let healthy: Vec<u16> = counts
        .keys()
        .copied()
        .filter(|status| *status < 300)
        .collect();

    let text = String::from("状态 200");
    println!("{healthy:?} 字节 {} 字符 {}", text.len(), text.chars().count());
}
```

---

## 9. 模块与可见性

| 语法 | 含义 |
|---|---|
| `mod helper { ... }` | 内联子模块 |
| `mod helper;` | 从 `helper.rs` 加载 |
| `pub` | 对外可见 |
| `pub(crate)` | 仅本 crate |
| `pub(super)` | 仅父模块 |
| `use crate::store::SqliteRepository;` | 引入作用域 |
| `pub use` | 重导出（缩短外部路径） |
| `crate::` / `super::` / `self::` | 绝对 / 父 / 当前 |
| `#[cfg(test)] mod tests` | 只在测试构建时编译 |

可见性是「向下」的：父模块看不到子模块的私有项，反过来可以。

---

## 10. 并发原语

| 需求 | 单线程 | 多线程 | 异步 |
|---|---|---|---|
| 共享所有权 | `Rc<T>` | `Arc<T>` | `Arc<T>` |
| 内部可变性 | `RefCell<T>` | `Mutex<T>` / `RwLock<T>` | `tokio::sync::Mutex` |
| 传递消息 | — | `std::sync::mpsc` | `tokio::sync::mpsc` |
| 线程/任务 | — | `thread::spawn` / `thread::scope` | `tokio::spawn` |
| 原子计数 | `Cell<u32>` | `AtomicUsize` | `AtomicUsize` |

| trait | 含义 |
|---|---|
| `Send` | 所有权可以安全地移到另一个线程 |
| `Sync` | `&T` 可以安全地被多个线程共享 |

**提问顺序**：能不能独占（直接 move）→ 能不能传消息（channel）→ 才考虑共享状态（`Arc<Mutex<T>>`）。

---

## 11. 属性速查

| 属性 | 作用 |
|---|---|
| `#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]` | 自动实现常见 trait |
| `#[must_use]` | 返回值被忽略时产生警告 |
| `#[cfg(test)]` | 只在测试构建时编译 |
| `#[test]` / `#[should_panic]` / `#[ignore]` | 测试标记 |
| `#[tokio::test]` / `#[tokio::test(start_paused = true)]` | 异步测试；暂停时钟 |
| `#[arg(long, default_value_t = 5_000)]` | Clap 参数声明 |
| `#[command(subcommand)]` / `#[derive(Parser)]` | Clap 命令结构 |
| `#[expect(clippy::lint)]` | 预期某个 lint，未触发反而报错 |
| `#[allow(...)]` | 关闭某个 lint |
| `#[instrument]` | tracing 自动建 span |

---

## 12. 常用标准库 trait

| trait | 提供什么 | 什么时候自己实现 |
|---|---|---|
| `Display` | `{}` 格式化 | 面向用户的输出 |
| `Debug` | `{:?}` 格式化 | 几乎总是 derive |
| `Clone` / `Copy` | 显式/隐式复制 | 拥有资源的类型只实现 `Clone` |
| `Default` | `Default::default()` | 有合理默认值时 |
| `PartialEq` / `Eq` | `==` | 需要比较时 |
| `Hash` | 可作 `HashMap` 键 | 键类型 |
| `From` / `TryFrom` | 类型转换 | 领域转换（`?` 依赖它） |
| `Iterator` | 迭代协议 | 自定义迭代器 |
| `Drop` | 析构钩子 | 需要释放非内存资源时 |
| `Error` | 标准错误接口 | 自定义错误类型 |
| `Deref` | `*` 解引用 | **谨慎**：容易滥用成继承 |
| `AsRef` / `Borrow` | 借用转换 | 想让 API 接受多种形式时 |

---

## 相关章节

| 想深入 | 读 |
|---|---|
| 所有权与借用 | [所有权：移动、借用与可变借用](../guided/ownership.md) |
| `Option` 与 `Result` | [用 Option 与 Result 表达分支](../core/result-option.md) |
| 枚举与集合 | [值、枚举、集合与模块边界](../core/values-collections-modules.md) |
| 生命周期、泛型、闭包 | [生命周期、泛型与闭包](../core/lifetimes-generics-closures.md) |
| trait 与测试 | [用 trait 建立可替换接缝](../core/traits-and-tests.md) |
| 并发 | [线程、消息与共享状态](../async/threads-channels-state.md) |
| 异步 | [Future、任务与运行时](../async/future-runtime.md) |
| 编译器报错 | [配方索引](../recipes/index.md) |
