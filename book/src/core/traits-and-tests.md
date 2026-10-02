# 用 trait 建立可替换接缝

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 让内存目标列表共用读取方式 | trait、泛型、借用、集成测试 | 2.5 小时 | 同步 `TargetSource` 接口 + 契约测试 |

> **预计 2.5 小时**。这一章做两件事，而且它们是同一件事的两面：**用 trait 定义接缝**，**用测试锁住接缝**。
>
> 这里暂时不出现 `async`、`Send + Sync` 或数据库。先用一个同步 trait 学会：调用者只依赖能力，具体实现藏在后面。

---

## 1. 可测试性是设计出来的，不是测出来的

先看一个很常见、也很难测的设计：

```rust,ignore
pub async fn latest_status(database_url: &str, target_id: i64) -> Result<Option<u16>, sqlx::Error> {
    let pool = sqlx::SqlitePool::connect(database_url).await?;
    // ... 直接查数据库
}
```

想测这个函数，你必须：准备一个数据库文件、保证它处于某个状态、跑完再清理。测试因此变慢、变脆，最后被标上 `#[ignore]`——**没人再跑它**。

把「怎么存」抽出来之后，同一个逻辑变成了可测的：

```rust,ignore
pub async fn latest_status<R: MonitorRepository>(repository: &R, target_id: i64)
    -> Result<Option<u16>, StoreError>
{
    Ok(repository.latest_result(target_id).await?.map(|result| /* ... */))
}
```

测试传一个内存实现，生产传 SQLite 实现，**被测试的代码一行都不用改**。这就是接缝（seam）的全部意义：**在不需要知道实现的前提下，替换掉实现。**

项目里已经有一处这样的证据：

```sh
cargo test -p monitor-store --test contract
```

`projects/monitor-store/tests/contract.rs` 用同一个泛型断言函数跑两个适配器——内存和 SQLite 必须表现完全一致。**如果接缝只存在于类型签名里、却没有契约测试，那它就不是接缝，只是多了一层。**

---

## 2. trait 能做的三件事

很多人从 Java/C# 过来，会把 `trait` 直接对应成 `interface`。它们确实像，但 Rust 的 trait 多做两件事：

| 能力 | 说明 | 例子 |
|---|---|---|
| 定义能力 | 只写方法签名，实现方补全 | `fn latest(&self) -> Option<&Target>;` |
| 提供默认实现 | 有签名也可以有方法体，实现方可覆盖 | `fn has_any(&self) -> bool { self.latest().is_some() }` |
| 作为约束 | 出现在泛型参数上，表达「需要什么能力」 | `fn f<T: Display>(value: T)` |

还有两个概念在 Rust 里很常见，但在「接口」的世界里没有对应物：

- **关联类型**：`type Item;`——把「和实现绑定的类型」也交给实现方决定。`Iterator` 的 `Item` 就是关联类型，而不是泛型参数，因为一个类型只应该有一种「迭代出来的东西」。
- **blanket impl**：`impl<T: Display> MyTrait for T`——为所有满足条件的类型统一实现。标准库用它把 `ToString` 自动送给每个 `Display`。

```rust,editable
trait Source {
    type Item;

    fn latest(&self) -> Option<&Self::Item>;

    // 默认实现：实现方不必重复写
    fn has_any(&self) -> bool {
        self.latest().is_some()
    }
}

impl Source for Vec<String> {
    type Item = String;
    fn latest(&self) -> Option<&String> {
        self.last()
    }
}

fn main() {
    let values = vec![String::from("a")];
    println!("有内容吗 = {}", values.has_any());
    println!("最新一条 = {:?}", values.latest());
}
```

关联类型与泛型参数的区别，判断标准只有一句：**「一个类型能有几种实现方式」**。一个 `Vec<String>` 只应该有一种 `Source` 实现，所以 `Item` 是关联类型；而 `From<T>` 有无数种源类型，所以 `T` 是泛型参数。
---

## 3. 一个完整的接缝：从具体类型到 trait

```rust,editable
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
    let targets = vec![Target { name: String::from("Rust") }];
    assert_eq!(latest_name(&targets), Some("Rust"));
}
```

先预测两个问题：

1. 为什么 `latest` 返回 `Option<&Target>`，而不是 clone 一份？
2. `&impl TargetSource` 与 `&dyn TargetSource` 各自把什么决定留给编译期、什么留给运行期？

第一个问题的答案是借用章节的结论：调用方可能只是要看一眼名字，不该为一次读取付出堆分配。

### 3.1 `impl Trait` 还是 `&dyn Trait`

```rust,editable
#[derive(Debug)]
struct Target { name: String }

trait TargetSource {
    fn latest(&self) -> Option<&Target>;
}

impl TargetSource for Vec<Target> {
    fn latest(&self) -> Option<&Target> { self.last() }
}

// 静态分派：编译期为每个具体类型生成一份函数
fn latest_name_static(source: &impl TargetSource) -> Option<&str> {
    source.latest().map(|target| target.name.as_str())
}

// 动态分派：运行期通过虚表找到实现
fn latest_name_dynamic(source: &dyn TargetSource) -> Option<&str> {
    source.latest().map(|target| target.name.as_str())
}

fn main() {
    let targets = vec![Target { name: String::from("Rust") }];
    println!("{:?}", latest_name_static(&targets));
    println!("{:?}", latest_name_dynamic(&targets));
}
```

| | `&impl Trait`（泛型） | `&dyn Trait` |
|---|---|---|
| 具体类型何时确定 | 编译期 | 运行期 |
| 能否内联 | 能 | 不能（一次间接调用） |
| 二进制体积 | 每个类型一份代码 | 一份代码 |
| 能否在同一个集合里混放 | 不能 | 能（`Vec<Box<dyn T>>`） |
| 需要对象安全 | 不需要 | **需要** |
| 调用方看到的类型 | 具体类型（被隐藏） | 统一的不透明类型 |

**默认选泛型**：它更快，而且没有对象安全的限制。只有在「运行期需要在同一个变量或集合里放不同的实现」时才用 `dyn`——比如插件注册表、`Vec<Box<dyn Handler>>`。

### 3.2 孤儿规则：为什么不能给 `Vec<T>` 加方法

想给标准库类型实现一个 trait，有时会撞上这堵墙：

```rust,compile_fail
impl std::fmt::Display for Vec<u8> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} bytes", self.len())
    }
}

fn main() {
    println!("{}", vec![1_u8, 2]);
}
```

```text
error[E0117]: only traits defined in the current crate can be implemented for
              types defined outside of the crate
  |
1 | impl std::fmt::Display for Vec<u8> {
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^-------
  |                            |
  |                            `Vec` is not defined in the current crate
  |
  = note: impl doesn't have any local type before any uncovered type parameters
  = note: define and implement a trait or new type instead
```

「孤儿规则」要求：**trait 和类型里至少有一个是你这个 crate 定义的**。它保护的是全局一致性——否则两个 crate 可以给同一个类型实现同一个 trait，而第三个 crate 同时依赖它们时无从选择。

标准解法是 newtype：包一层自己的类型，然后给它实现。

```rust,editable
struct Names(Vec<String>);

impl std::fmt::Display for Names {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} names", self.0.len())
    }
}

fn main() {
    let names = Names(vec![String::from("Rust"), String::from("Example")]);
    println!("{names}");
}
```

newtype 还有第二个好处：`Names` 与 `Vec<String>` 是两个类型，编译器不会让你把任意 `Vec<String>` 当成 `Names` 用——**这本身就是一种校验**。

---

## 4. 对象安全：一个会一直跟到 Web 阶段的决定

`dyn Trait` 只接受「对象安全」的方法：

- 不能有泛型参数（`fn get<T>(&self, key: T)` 不行）；
- 不能返回 `Self`（除非加了 `where Self: Sized`）；
- 不能是 `async fn`（它的返回类型是不透明的 future）。

这解释了本阶段练习里那个看着别扭的签名：返回 `Option<&Target>` 而不是 `Target`，恰好也是对象安全的——`&Target` 是具体类型，不涉及 `Self`。

到异步存储阶段，这条限制会真正咬人：`async fn` 既不是对象安全的，它的 future 也不满足 `Send`，而 axum 的 handler 必须 `Send`。三条出路：

| 写法 | 装箱 | 能用于 `dyn` | 结论 |
|---|---|---|---|
| `async fn` in trait | 否 | 否 | 本项目不能用 |
| `#[async_trait]` | 每次调用一次 | 是 | 可用，但有代价 |
| `-> impl Future + Send` | 否 | 否 | 本项目选它 |

选了第三条，就意味着放弃 `dyn`，把路由改成对 `R: MonitorRepository` 泛型。完整推导见 `docs/adr/0017` 与[对象安全配方](../recipes/object-safety.md)。

**所以这一节的完成标准里，对象安全不是背景知识，而是你以后每次写 `dyn` 都要先算的一笔账。**
---

## 5. 用测试锁住接缝

有了接缝却不测试，等于什么都没做——你只是多了一层间接调用。这一节把测试的四种形态按「测试目标」分开讲。

### 5.1 单元测试：和代码放在同一个文件里

```rust,editable
fn parse_port(text: &str) -> Result<u16, std::num::ParseIntError> {
    text.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_numeric_port() {
        assert_eq!(parse_port("8080").ok(), Some(8080));
    }

    #[test]
    fn rejects_a_non_numeric_port() {
        assert!(parse_port("http").is_err());
    }
}

fn main() {
    println!("{:?}", parse_port("8080"));
}
```

三个要点：

- `#[cfg(test)]` 让整个模块**只在 `cargo test` 时编译**，不进入生产二进制。
- `use super::*;` 让测试模块能看到父模块的私有项——这是单元测试能测「内部实现」的原因。
- 测试函数用 `#[test]` 标记，断言用 `assert!` 与 `assert_eq!`。

**单元测试测的是「这一个小函数的行为」。** 它不该启动服务器、连数据库或读写文件。

### 5.2 集成测试：`tests/` 目录，只能看到公开 API

```text
projects/monitor-store/
├── src/
│   └── lib.rs
└── tests/
    ├── contract.rs      ← 每个文件都被编译成一个独立的 crate
    ├── in_memory.rs
    └── sqlite.rs
```

集成测试文件是**独立 crate**，只能 `use monitor_store::...`——看不到任何 `pub(crate)` 或私有项。这个限制是有价值的：它强制你只测试**对外承诺的行为**，而不是实现细节。重构内部实现时，集成测试不该失败。

### 5.3 契约测试：一次断言，跑遍所有实现

这是本章最重要的一节。项目的 `tests/contract.rs` 是这么写的：

```rust,ignore
#[tokio::test]
async fn in_memory_rejects_a_result_for_a_different_target() {
    assert_mismatched_result_is_rejected(&InMemoryRepository::new()).await;
}

#[tokio::test]
async fn sqlite_rejects_a_result_for_a_different_target() {
    let repository = SqliteRepository::connect("sqlite::memory:").await.unwrap();
    assert_mismatched_result_is_rejected(&repository).await;
}

/// 参数曾是 &dyn MonitorRepository；现在写成泛型，因为原生 async trait
/// 方法不是对象安全的——这正是这个 crate 有意做的取舍。
async fn assert_mismatched_result_is_rejected<R: MonitorRepository>(repository: &R) {
    // ... 断言两种实现表现完全一致
}
```

这个形状值得记住：**断言写成泛型函数，每个实现各写一个三行的 `#[test]` 调用它。** 新增一个适配器时，你只要加一个测试函数，就自动获得了全部契约检查。

```rust,editable
trait Store {
    fn save(&mut self, value: u16);
    fn latest(&self) -> Option<u16>;
}

#[derive(Default)]
struct Memory {
    values: Vec<u16>,
}

impl Store for Memory {
    fn save(&mut self, value: u16) {
        self.values.push(value);
    }
    fn latest(&self) -> Option<u16> {
        self.values.last().copied()
    }
}

/// 所有 Store 实现都必须满足的契约。
fn contract<S: Store + Default>() {
    let mut store = S::default();
    assert_eq!(store.latest(), None, "新存储应该是空的");

    store.save(200);
    assert_eq!(store.latest(), Some(200));

    store.save(404);
    assert_eq!(store.latest(), Some(404), "latest 返回最后写入的值");
}

#[test]
fn memory_satisfies_the_contract() {
    contract::<Memory>();
}

fn main() {}
```

### 5.4 测试名该怎么起

项目的命名风格是完整的句子，读起来像规格说明：

| 不好的名字 | 好的名字 |
|---|---|
| `test_new` | `learner_gets_a_specific_error_for_a_blank_target_name` |
| `test_parse_2` | `rejects_a_name_that_is_only_whitespace` |
| `test_contract` | `sqlite_rejects_a_result_for_a_different_target` |

判断标准：**测试失败时，光看名字能不能知道哪个承诺被打破了？** 能，就是好名字。

### 5.5 三种特殊的测试形态

```rust,editable
fn parse_port(text: &str) -> Result<u16, std::num::ParseIntError> {
    text.parse()
}

/// 返回 Result 的测试可以用问号运算符，失败时打印错误而不是 panic 消息。
#[test]
fn accepts_a_numeric_port() -> Result<(), std::num::ParseIntError> {
    let port = parse_port("8080")?;
    assert_eq!(port, 8080);
    Ok(())
}

/// 期望 panic 时用 should_panic，并尽量断言消息片段。
#[test]
#[should_panic(expected = "must be non-empty")]
fn rejects_an_empty_name() {
    let name = "";
    assert!(!name.is_empty(), "must be non-empty");
}

/// 需要人工完成的检查点用 ignore 标记：CI 会跳过它，但它仍然会被编译。
#[test]
#[ignore = "learning checkpoint: remove this attribute before solving"]
fn learner_moves_a_result_into_history() {
    todo!("学习者自己实现")
}

fn main() {}
```

第三种形态在项目里真实存在：`monitor-domain/tests/learner_checkpoint.rs` 用 `#[ignore]` 标记「留给学习者解决的检查点」。**`#[ignore]` 的测试仍然会被编译**，所以它不会悄悄腐坏。

### 5.6 确定性：测试不依赖网络，也不依赖真实等待

```rust,editable
use std::io::{Read, Write};
use std::net::TcpListener;

/// 启动一个只服务一次请求的本地服务器，返回它的地址。
/// 端口用 0 让操作系统分配，避免测试之间抢端口。
fn serve_once(response: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("应该能绑定本地端口");
    let address = listener.local_addr().expect("应该能拿到地址");

    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("应该接受连接");
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request);
        let _ = stream.write_all(response.as_bytes());
    });

    format!("http://{address}")
}

fn main() {
    let url = serve_once("HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n");
    println!("本地测试服务器: {url}");
}
```

两条项目规则，来自 `docs/adr/0011`：

1. **不访问公网**：用 `127.0.0.1:0` 上的临时服务器。开发机器上配了公司代理时，还要用类似 `MONITOR_DISABLE_PROXY` 的手段绕开它，否则本地请求会被代理拦走。
2. **不依赖真实等待**：验证超时和退避不要先 sleep 再断言——在繁忙的 CI 上会随机失败。用 `tokio::time::pause()` 或 `#[tokio::test(start_paused = true)]` 把时间变成可控变量。

> 一条实战经验：**当你发现某个测试必须靠 sleep 才能通过，那说明被测代码缺少一个可以观察的事件、或一个可以注入的时钟。** 这通常不是测试的问题，是设计的问题。

### 5.7 文档测试：写在注释里的测试

文档注释里的代码块会被 `cargo test` 编译并运行。写出来是这样（例子用四空格缩进，因为文档注释里再套一层围栏会很别扭）：

```text
/// 把秒数格式化成人类可读的时长。
///
/// 例子：
///
///     assert_eq!(describe(90), "1 分 30 秒");
///
pub fn describe(seconds: u64) -> String {
    if seconds < 60 {
        return format!("{seconds} 秒");
    }
    format!("{} 分 {} 秒", seconds / 60, seconds % 60)
}
```

好处是**文档和代码不可能不一致**——例子跑不起来，`cargo test` 就会失败。代价是文档测试要单独编译一次，所以只给「值得当例子」的 API 写。

---

## 6. 深水区

### 6.1 测试替身的三种形态

| 形态 | 写法 | 适合 |
|---|---|---|
| 手写内存实现 | `InMemoryRepository` 实现同一个 trait | 大多数情况；行为可预测 |
| 泛型参数 | `fn f<S: Store>(store: &S)` | 生产代码只需要一种实现 |
| trait 对象 | `fn f(store: &dyn Store)` | 运行期需要切换，或要放进集合 |

项目用的是第一种加第二种：内存实现是替身，泛型参数是注入方式。

### 6.2 什么时候不该造接缝

```rust,ignore
trait StringFormatter { fn format(&self, value: &str) -> String; }
```

如果一个 trait 只有一个实现、不隐藏任何复杂度、也不会被替换，它就是纯粹的负担：多一个文件、多一层跳转、读者还得跟着跳一次。**判断标准是「它隐藏了什么」，不是「它看起来解耦了吗」。** 项目的 ADR 0016 把这条写成了决策：某项功能只被一个应用使用、也没有隐藏显著复杂度时，不为「架构整齐」继续拆 crate。

### 6.3 别用 feature 开关测试代码

`cfg(test)` 只在测试构建时开启；`cfg(feature = "x")` 由使用者在 `Cargo.toml` 里选择。两者都是条件编译，但用途不同——用 feature 开关测试代码，会把「跑没跑测试」变成一个配置问题。

---

## 7. 常见误解

| 误解 | 准确说法 |
|---|---|
| 「trait 就是 interface」 | Rust 的 trait 还能有默认实现、关联类型和 blanket impl，表达力更强。 |
| 「dyn 一定慢」 | 代价是一次虚表跳转和无法内联。只有热路径上才值得为此改写设计。 |
| 「集成测试要启动真实服务」 | 用 `127.0.0.1:0` 上的临时服务器，零外部依赖、不抢端口。 |
| 「测试代码不用管 lint」 | CI 用 `--all-targets` 跑 clippy 并 `-D warnings`，测试代码同样要干净。 |
| 「接口抽出来就自动可测」 | 没有契约测试的接缝只是多一层间接；可替换性要用测试证明。 |
| 「异步测试 sleep 一下就好」 | 应使用暂停时钟或事件通知，否则 CI 会随机失败。 |

---

## 8. 练习与自测

### 练习

```sh
cd exercises
rustlings run 05_repository
```

先让练习失败，再按「方向 → API → 形状」三层提示修复。

然后在项目里做一件事：打开 `projects/monitor-store/tests/contract.rs`，回答「如果明天新增一个 PostgreSQL 适配器，我需要写多少测试代码？」——答案应该是「一个三行的 `#[tokio::test]` 加一行泛型调用」。

### 自测清单（能全部做到才算掌握）

1. 说明 `impl Trait` 与 `dyn Trait` 各自的取舍，并说出本项目在哪一步放弃了 `dyn`、为什么。
2. 写出孤儿规则的两种合法实现方式（trait 属于本 crate / 类型属于本 crate）。
3. 解释关联类型与泛型参数的区别，判断标准是什么。
4. 写一个契约测试：一个泛型断言函数，加两个实现各一次调用。
5. 说出单元测试与集成测试在「能看见什么」上的区别，以及这个区别为什么有价值。
6. 用 `127.0.0.1:0` 写一个只服务一次的本地 HTTP 服务器，并说明端口为什么用 0。
7. 解释为什么「必须 sleep 才能通过」的测试通常说明设计有问题。
8. 给出一个「不该造接缝」的例子，并说明判断标准。

完成标准：能在不 clone `Target` 的情况下替换数据来源，并解释对象安全为什么会影响 `dyn Trait` 的方法形状。
