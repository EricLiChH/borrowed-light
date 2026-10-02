# 值、枚举、集合与模块边界

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 把原始输入变成可遍历的领域数据 | expression、struct、enum、match、slice、Vec、HashMap、module | 3 小时 | 监测目标与结果汇总 |

> **预计 3 小时**。这一章是「语法密度」最高的一章：表达式、枚举、集合、模块四件事，每一件都会在你之后的每一行代码里出现。

本章的目标不是记住 API，而是建立四个判断力：

1. 什么时候该用 `enum` 而不是「布尔值 + 可空字段」；
2. 什么时候该用 `Vec`、什么时候该用 `&[T]`；
3. 什么时候该用 `String`、什么时候该用 `&str`；
4. 什么该藏在模块里面。

---

## 1. 一切皆表达式，分号是「丢弃」

在 Rust 里，`if`、`match`、代码块都是**表达式**：它们有值。分号的作用不是「结束一行」，而是**把值丢掉，让整个表达式变成 `()`**。

```rust,editable
fn main() {
    let code = if true { 1 } else { 2 };
    println!("code = {code}");

    // loop 也能返回值：break 后面可以带一个值
    let mut attempts = 0;
    let settled = loop {
        attempts += 1;
        if attempts == 3 {
            break attempts * 10;
        }
    };
    println!("settled = {settled}");
}
```

这条规则解释了新手最常见的编译错误之一。下面这个函数想返回一个 `String`，但 `else` 分支的最后多了一个分号：

```rust,compile_fail
fn label(status: u16) -> String {
    if status == 200 {
        String::from("healthy")
    } else {
        String::from("unknown");
    }
}

fn main() {
    println!("{}", label(200));
}
```

```text
error[E0308]: mismatched types
   |
 4 |       } else {
   |  ____________^
 5 | |         String::from("unknown");
   | |                                - help: remove this semicolon to return this value
 6 | |     }
   | |_____^ expected `String`, found `()`
```

`expected `String`, found `()`` 这句报错的完整含义是：**这个分支的值被分号丢掉了，所以它没有值**。看到它，第一件事就是找分号。

> 反过来也是一条实用的技巧：函数的最后一行**不加分号**才是返回值。`return` 关键字在 Rust 里用得很少，只在提前退出时才需要。

---

## 2. 绑定、可变性与遮蔽

```rust,editable
fn main() {
    // 默认不可变
    let name = String::from("Rust");

    // 遮蔽（shadowing）：用同名绑定覆盖上一个，可以换类型
    let name = name.len();
    let name = format!("名字长度 {name}");

    println!("{name}");

    // 需要原地修改才加 mut
    let mut counter = 0;
    counter += 1;
    println!("counter = {counter}");
}
```

`let` 与 `let mut` 的区别是「能不能改」，**遮蔽**与 `mut` 的区别是「是不是同一个变量」：遮蔽产生一个全新的绑定，可以换类型、可以重新借用，旧值在新绑定可见的范围内不可访问（如果它被移动了，就在这里析构）。

另外两个容易混淆的写法：

| 写法 | 含义 | 什么时候用 |
|---|---|---|
| `let _ = value;` | **立刻丢弃**这个值（`Drop` 马上运行） | 明确表示「我不要这个结果」 |
| `let _guard = value;` | 绑定到带下划线的名字，值**活到作用域结束** | 持有锁、守卫、订阅等有副作用生命周期的值 |

这个区别在并发代码里会咬人：写 `let _ = mutex.lock()` 会在语句结束时就释放锁，而你往往想要的是 `let _guard = mutex.lock()`。

常量与静态量是另外一回事：

```rust,editable
const DEFAULT_TIMEOUT_MS: u64 = 5_000;          // 编译期内联，没有地址
static APP_NAME: &str = "borrowed-light";       // 有固定地址，全局唯一

fn main() {
    println!("{APP_NAME} 默认超时 {DEFAULT_TIMEOUT_MS} 毫秒");
}
```

`const` 是「把值抄进每个使用点」，`static` 是「全局只有一份」。需要可变全局状态时要配合同步原语（`Mutex`、`OnceLock`），这也是编译器会在你写 `static mut` 时警告的原因。
---

## 3. 用 `enum` 建模状态

原始输入迟早要变成有约束的领域数据。两种建模方式：

```rust,ignore
// 松散的：三个字段，靠约定保证「失败时 status 是 None」
struct LooseResult {
    reachable: bool,
    status: Option<u16>,
    reason: Option<String>,
}

// 紧凑的：类型本身保证「要么有状态码，要么有原因」
enum CheckOutcome {
    Reachable { status: u16 },
    Unreachable { kind: CheckFailureKind, reason: String },
}
```

第一种的错误要靠**约定**避免：任何人只要写错一个字段组合（`reachable: true` 且 `status: None`），数据就自相矛盾。第二种根本表达不出矛盾状态——这就是「让非法状态无法表示」。这条原则在本项目的演进里出现过三次：`CheckOutcome`、`StoreError`、`ApiError`。

### 模式匹配的完整语法

```rust,editable
#[derive(Debug)]
enum Outcome {
    Reachable(u16),
    Unreachable(String),
}

fn label(outcome: &Outcome) -> String {
    match outcome {
        // 范围模式
        Outcome::Reachable(200..=299) => String::from("healthy"),
        // @ 绑定：既匹配范围，又把值绑到名字上
        code @ Outcome::Reachable(300..=399) => format!("redirected: {code:?}"),
        // 守卫（guard）：模式之后再加条件
        Outcome::Reachable(status) if *status >= 500 => format!("server error {status}"),
        Outcome::Reachable(status) => format!("responded {status}"),
        Outcome::Unreachable(reason) => format!("unreachable: {reason}"),
    }
}

fn main() {
    println!("{}", label(&Outcome::Reachable(204)));
    println!("{}", label(&Outcome::Reachable(500)));
    println!("{}", label(&Outcome::Unreachable(String::from("timeout"))));
}
```

模式能做的事，比 `switch` 多得多：

| 语法 | 例子 | 含义 |
|---|---|---|
| 字面量 | `200` | 精确匹配 |
| 范围 | `200..=299` | 闭区间 |
| 或 | `400 \| 404 \| 500` | 任一匹配即可 |
| 元组解构 | `(name, Some(url))` | 同时匹配多个位置 |
| 结构体解构 | `Target { url, .. }` | 按字段名匹配，`..` 忽略其余 |
| 切片解构 | `[first, rest @ ..]` | 首元素 + 剩余部分 |
| 绑定 | `status` | 把匹配到的值绑到名字 |
| 绑定 + 匹配 | `code @ 200..=299` | 两者都要 |
| 守卫 | `n if n > 0` | 模式之外再加条件 |
| 通配 | `_` | 不关心这个位置 |

**穷尽性由编译器强制。** 漏掉一个变体不会变成运行期 `default`，而是编译错误：

```rust,compile_fail
enum Outcome {
    Reachable(u16),
    Unreachable(String),
}

fn label(outcome: &Outcome) -> &'static str {
    match outcome {
        Outcome::Reachable(200..=299) => "healthy",
        Outcome::Reachable(_) => "responded",
    }
}

fn main() {
    println!("{}", label(&Outcome::Reachable(204)));
}
```

```text
error[E0004]: non-exhaustive patterns: `&Outcome::Unreachable(_)` not covered
  |
7 |     match outcome {
  |           ^^^^^^^ pattern `&Outcome::Unreachable(_)` not covered
  |
note: `Outcome` defined here
  |
3 |     Unreachable(String),
  |     ----------- not covered
  |
help: ensure that all possible cases are being handled by adding a match arm
      with a wildcard pattern or an explicit pattern as shown
  |
9 ~         Outcome::Reachable(_) => "responded",
10 ~         `&Outcome::Unreachable(_)` => todo!(),
```

注意编译器说的是 `&Outcome::Unreachable(_)` 而不是 `Outcome::Unreachable(_)`：因为 `outcome` 是引用，匹配引用时 Rust 会自动以「引用模式」工作（match ergonomics），绑定的字段也自动是引用——所以上面例子里 `status` 的类型是 `&u16`，比较时要写 `*status`。

### 除了 `match` 之外的四个工具

```rust,editable
fn main() {
    let candidate: Option<u16> = Some(200);

    // if let：只关心一个分支
    if let Some(status) = candidate {
        println!("if let: {status}");
    }

    // let else：不匹配就离开当前作用域（绑定在之后继续可用）
    let Some(status) = candidate else {
        println!("let else: 没有值，提前退出");
        return;
    };
    println!("let else 之后仍可用: {status}");

    // let chains（Rust 2024 起稳定）：多个条件连着写
    if let Some(code) = candidate && code < 300 {
        println!("let chains: {code} 是成功状态");
    }

    // while let：一直匹配到不匹配为止
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        println!("while let 弹出 {top}");
    }
}
```

选择依据很简单：**`match` 用于「穷尽地把所有情况分类」，其余四个用于「只关心一种情况」。** 当你发现 `match` 里有一堆 `_ => {}`，那大概率应该写成 `if let`。

---

## 4. `Copy`、move 与 drop

`u16` 实现 `Copy`，赋值后两边都能继续用；`String` 管理堆内存，默认会 move，离开最后一个所有者的作用域时执行 drop。判断标准不是「在栈上还是堆上」，而是：**这个类型实现 `Copy` 了吗？接口接管 `T` 了吗？**

```rust,compile_fail
fn main() {
    let name = String::from("Rust");
    let moved = name;
    println!("{name} -> {moved}");
}
```

两种类型的差别可以总结成一张表：

| | `Copy` 类型（`u16`、`bool`、`&T`） | 拥有资源的类型（`String`、`Vec<T>`） |
|---|---|---|
| `let b = a;` 之后 | 两个都能用 | 只有 `b` 能用 |
| 传参给 `fn f(x: T)` | 原变量仍可用 | 原变量被移动 |
| 需要显式复制吗 | 不需要 | 需要 `.clone()` |
| 能实现 `Drop` 吗 | 不能 | 可以 |
| 典型实现 | `#[derive(Copy, Clone)]` | `#[derive(Clone)]` |
---

## 5. 三个集合，三组坑

### `Vec<T>`：索引会 panic，`get` 不会

```rust,editable
fn main() {
    let mut statuses = vec![200_u16, 404];
    statuses.push(503);

    // 索引：越界就 panic
    println!("第一个 = {}", statuses[0]);

    // get：越界返回 None
    println!("第 99 个 = {:?}", statuses.get(99));

    // 保留满足条件的元素（原地、无额外分配）
    statuses.retain(|status| *status < 500);
    println!("过滤后 = {statuses:?}");
}
```

**规则：当下标来自外部输入时用 `get`，来自你自己刚做过的检查时用索引。** 这不是风格问题——索引越界是 panic，而 panic 在 Web 服务里意味着一个请求把整个连接搞崩（或者让整个进程退出，取决于配置）。

从 `Vec` 里「取出」一个元素是移动，直接赋值给变量会被拒绝：

```rust,compile_fail
fn main() {
    let targets = vec![String::from("https://example.com")];
    let first = targets[0];
    println!("{first}");
}
```

```text
error[E0507]: cannot move out of index of `Vec<String>`
  |
3 |     let first = targets[0];
  |                 ^^^^^^^^^^ move occurs because value has type `String`,
  |                            which does not implement the `Copy` trait
  |
help: consider borrowing here
  |
3 |     let first = &targets[0];
  |                 +
```

原因很实际：`Vec` 里的元素是**连续排列**的，挖走中间一个会在数组里留下空洞。要真的拿走，得告诉别人怎么补：`swap_remove`（O(1)，打乱顺序）、`remove`（O(n)，保序），或者 `pop`（从尾部拿走）。

三种迭代方式的区别值得背下来：

| 写法 | 产出 | 集合之后还能用吗 | 什么时候用 |
|---|---|---|---|
| `for item in &vec` | `&T` | 能（只读借用） | 只读遍历，最常用 |
| `for item in &mut vec` | `&mut T` | 能（独占借用） | 原地修改每个元素 |
| `for item in vec` | `T` | **不能**（被消耗） | 要把元素转移走 |

### `String` 与 `&str`：一个是所有者，一个是视图

- `String`：拥有堆上的 UTF-8 字节缓冲区，可以增长。
- `&str`：对一段 UTF-8 字节的**借用视图**，包含指针和长度，不知道谁拥有它。

函数参数应该优先用 `&str`：它同时接受 `&String`（自动解引用）和字符串字面量，而且不强迫调用方交出所有权。

```rust,editable
fn is_https(url: &str) -> bool {
    url.starts_with("https://")
}

fn main() {
    let owned = String::from("https://example.com");
    println!("{}", is_https(&owned));       // &String -> &str
    println!("{}", is_https("https://x"));  // 字面量本来就是 &str
}
```

**为什么不能按整数索引？** 因为 UTF-8 是变长编码，第 n 个「字节」不一定是第 n 个「字符」。Rust 干脆禁止这种歧义：

```rust,compile_fail
fn main() {
    let url = String::from("https://example.com");
    println!("{}", url[0]);
}
```

```text
error[E0277]: the type `str` cannot be indexed by `{integer}`
  |
3 |     println!("{}", url[0]);
  |                        ^ string indices are ranges of `usize`
  |
  = note: you can use `.chars().nth()` or `.bytes().nth()`
```

中英混排时这个区别非常直观：

```rust,editable
fn main() {
    let text = String::from("状态 200");
    println!("字节数 = {}", text.len());
    println!("字符数 = {}", text.chars().count());

    for (index, character) in text.char_indices() {
        println!("第 {index} 个字节处是 {character:?}");
    }
}
```

字符 `状` 占 3 个字节，所以下一个字符从字节 3 开始——`len()` 和 `chars().count()` 在这种情况下差了一倍。**凡是「长度」两个字出现在字符串上，先问是字节还是字符。**

拼接字符串的三种写法，代价不同：

| 写法 | 分配次数 | 什么时候用 |
|---|---|---|
| `format!("{a}{b}")` | 一次 | 组合多个值，最常用 |
| `a.push_str(b)` | 原地追加（可能需要扩容） | 循环里累积，配合 `with_capacity` |
| `a + &b` | 消耗 `a` 并复用它的缓冲区 | 一次性拼接，且不再需要 `a` |

### `HashMap`：用 `entry` 代替「先查再插」

```rust,editable
use std::collections::HashMap;

fn main() {
    let mut counts: HashMap<u16, usize> = HashMap::new();

    for status in [200_u16, 200, 404] {
        // 有就取出来，没有就插入默认值，然后加一
        *counts.entry(status).or_insert(0) += 1;
    }

    // 只在已存在时修改，不存在时插入
    counts
        .entry(200)
        .and_modify(|count| *count += 10)
        .or_insert(0);

    // 需要构造开销时才用 or_insert_with
    counts.entry(500).or_insert_with(|| 0);

    // 迭代顺序不保证，要稳定输出就得排序
    let mut pairs: Vec<_> = counts.iter().collect();
    pairs.sort_by_key(|(status, _)| **status);
    println!("{pairs:?}");
}
```

- `entry` 返回一个 `Entry`，把「查」和「插」合成一次哈希查找。手写 `if !map.contains_key(k) { map.insert(...) }` 是两次查找，而且在并发或借用紧张的地方更难写。
- `counts[&200]` 在键不存在时会 **panic**；`get(&200)` 返回 `Option`。和 `Vec` 的规则一样。
- 需要**有序**遍历时用 `BTreeMap`（按 key 排序，查找 O(log n)），需要「最新插入顺序」时再看看 `IndexMap`（生态 crate）。
- 键必须实现 `Eq + Hash`；`f64` 两者都没有，所以不能当键（用整数或字符串坐标代替）。
---

## 6. slice 是集合的借用窗口

```rust,editable
fn healthy_count(statuses: &[u16]) -> usize {
    statuses
        .iter()
        .filter(|&&status| (200..300).contains(&status))
        .count()
}

fn main() {
    let statuses = vec![200, 204, 404, 503];

    // Vec 能自动变成 slice
    println!("全部 = {}", healthy_count(&statuses));
    // 也能只传一段
    println!("前三个 = {}", healthy_count(&statuses[0..3]));
    // 数组同样可以
    println!("数组 = {}", healthy_count(&[200, 500]));
}
```

**函数签名里写 `&[T]`，不要写 `&Vec<T>`。** 理由和 `&str` 对 `String` 完全一样：`&[T]` 接受 `Vec<T>`、数组、以及任何切片的一部分，而且不暴露「你用的是哪种容器」。`&[T]` 之于 `Vec<T>`，就是 `&str` 之于 `String`。同理，参数用 `&T` 而不是 `&Box<T>`。

`.iter()` 先借用，再由 `filter`、`map`、`collect` 组合转换；普通循环更清楚时就用循环，这不是品味问题而是可读性问题。

---

## 7. 模块：把不变量藏在入口后面

```rust,editable
mod monitor {
    pub struct Target {
        name: String,
    }

    impl Target {
        /// 唯一的构造入口：空名字无法创建出 Target。
        pub fn new(name: &str) -> Option<Self> {
            (!name.trim().is_empty()).then(|| Self { name: name.into() })
        }

        pub fn name(&self) -> &str {
            &self.name
        }
    }
}

fn main() {
    let target = monitor::Target::new("Rust").unwrap();
    assert_eq!(target.name(), "Rust");

    // 字段是私有的，模块外无法检查后再构造：
    // let broken = monitor::Target { name: String::new() };
}
```

`name` 字段没有 `pub`，所以模块外**只能**经过 `new`。这就是「把不变量藏在入口后面」：`Target` 一定有一个非空名字，这个事实由可见性保证，而不是靠约定。

模块系统的几条规则，按使用频率排：

| 写法 | 含义 |
|---|---|
| `pub` | 对外（crate 外）可见 |
| `pub(crate)` | 只在本 crate 内可见 |
| `pub(super)` | 只对父模块可见 |
| `use crate::monitor::Target;` | 把路径引入当前作用域 |
| `super::` / `crate::` | 相对父模块 / 相对 crate 根 |
| `pub use` | 重导出：让外部看到更短、更稳定的路径 |
| `mod x;` | 从 `x.rs` 或 `x/mod.rs` 加载子模块 |

两条容易踩的规则：

- **模块的可见性是「向下」的**：父模块看不到子模块的私有项，子模块能看到父模块的私有项。
- **`mod` 不等于文件**。`mod helper { ... }` 写在文件里就是内联模块，`mod helper;` 才是「去读另一个文件」。测试用的 `#[cfg(test)] mod tests ` 就是内联模块。

---

## 8. 深水区

### 8.1 结构体更新语法会移动字段

```rust,editable
#[derive(Debug)]
struct Target {
    name: String,
    url: String,
    enabled: bool,
}

fn main() {
    let base = Target {
        name: String::from("Rust"),
        url: String::from("https://www.rust-lang.org"),
        enabled: true,
    };

    // 未列出的字段从 base 移动过来（enabled 是 Copy，此处不涉及）
    let disabled = Target { enabled: false, ..base };
    println!("{disabled:?}");

    // 取消下一行的注释，会看到 E0382：base 的字段已经被部分移动
    // println!("{base:?}");
}
```

`..base` 不是「复制一份再改」，而是「把没写的字段**移动**过来」。如果这是你想要的，很好；如果不是，就该先 `clone` 或者手工列出所有字段。**这个语法在含 `String`/`Vec` 的结构体上尤其容易造成意外的部分移动。**

### 8.2 `Cow`：可能借用、也可能拥有

有些函数「大多数时候不用改动输入」——这时返回 `&str` 借用不够灵活（有时必须返回新分配的值），返回 `String` 又总是要分配。`Cow`（clone-on-write）两者兼顾：

```rust,editable
use std::borrow::Cow;

fn ensure_https(input: &str) -> Cow<'_, str> {
    if input.starts_with("https://") {
        Cow::Borrowed(input)      // 没改，零分配
    } else {
        Cow::Owned(format!("https://{input}"))   // 改了，分配一次
    }
}

fn main() {
    let already = ensure_https("https://example.com");
    let fixed = ensure_https("example.com");

    // Deref 让 Cow<str> 直接用起来像 &str
    println!("{} / {}", already.len(), fixed.len());
    println!("{}", matches!(already, Cow::Borrowed(_)));
}
```

看到 `Cow<'_, str>` 就该意识到：「这个函数不保证不分配，但大多数情况下不分配」。

### 8.3 容量与重新分配

```rust,editable
fn main() {
    let mut naive = Vec::new();
    let mut prepared = Vec::with_capacity(3);

    for value in [1_u32, 2, 3] {
        naive.push(value);
        prepared.push(value);
    }

    println!("naive    len={} cap={}", naive.len(), naive.capacity());
    println!("prepared len={} cap={}", prepared.len(), prepared.capacity());
}
```

`len` 是元素个数，`capacity` 是**已经分配的空间**。`push` 超过容量时会发生重新分配（通常是翻倍）并把旧元素全部搬过去。当你**事先知道**要放多少个元素时，`with_capacity` 能省掉这几次搬运——这在热路径上是有意义的优化，但不要在没有测量之前到处加它。

### 8.4 `HashMap` 的哈希不是免费的

标准库的 `HashMap` 默认使用 SipHash：抗哈希碰撞攻击（HashDoS），代价是比 `FxHashMap`（生态 crate）慢。绝大多数业务代码不需要换；只有在剖析工具指出它确实是热点时，才考虑换哈希器。

---

## 9. 常见误解

| 误解 | 准确说法 |
|---|---|
| 「`String` 就是堆上的 `&str`」 | 一个是**所有者**，一个是**视图**。`&str` 还可以指向字面量、`String` 的一部分或 `'static` 数据。 |
| 「`Vec::len()` 就是容量」 | `len` 是元素个数，`capacity` 是已分配空间；两者不等时 `push` 可能触发重新分配。 |
| 「`HashMap` 按插入顺序遍历」 | 不是。顺序由哈希决定，可能每次运行都不同；要稳定输出就排序或用 `BTreeMap`。 |
| 「索引比 `get` 快，所以都用索引」 | 索引越界会 panic。下标来自外部输入时必须用 `get`。 |
| 「`mod` 一定对应一个文件」 | 内联 `mod x { ... }` 也是模块；`mod x;` 才去读文件。 |
| 「`..base` 是复制其余字段」 | 是**移动**。含非 `Copy` 字段时，`base` 之后可能不可用。 |

---

## 10. 练习与自测

### 练习

```sh
cd exercises
rustlings run 10_collections
rustlings run 11_modules
```

依次完成 `10_collections` 与 `11_modules`，然后在项目里回答一个问题：**`projects/monitor-domain/src/target.rs` 里哪些字段是私有的？为什么 `MonitorTarget::new` 必须是唯一的构造入口？**

### 自测清单（能全部做到才算掌握）

1. 解释 `if` 作为表达式与「加分号」的区别，并说出对应的错误码。
2. 说出 `let _` 与 `let _guard` 的区别，并说明它在锁上的后果。
3. 手写一个含「范围模式 + `@` 绑定 + 守卫」的 `match`。
4. 解释为什么 `&Outcome::Unreachable(_)` 会出现在 E0004 的报错文字里。
5. 说出 `Vec` 三种迭代方式（`&vec`、`&mut vec`、`vec`）各自产出什么类型。
6. 解释「`String` 不能按整数索引」的两个理由（一个关于编码，一个关于语义）。
7. 用 `entry` 写一个词频统计，并说明它比「先查再插」省了什么。
8. 给一个函数签名，说明为什么参数应该写 `&[T]` 而不是 `&Vec<T>`。

第 4 题答不上来时，回到第 3 节末尾关于 match ergonomics 的那段；那不是细节，它决定了你写 `*status` 还是 `status`。
