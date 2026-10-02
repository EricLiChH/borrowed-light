# 用 `Option` 与 `Result` 表达分支

> **预计 2 小时**。这一章是全书最"便宜"的一章：概念只有两个，但它们决定了你之后每一个函数的签名。

先预测：第二次查找没有结果时，函数应该返回空值、错误，还是直接终止程序？

```rust,editable
fn latest_status(history: &[u16]) -> Option<u16> {
    history.last().copied()
}

fn main() {
    println!("{:?}", latest_status(&[200, 204]));
    println!("{:?}", latest_status(&[]));
}
```

三种答案对应三种设计：

| 返回 | 含义 | 调用方必须做什么 |
|---|---|---|
| `Option<T>` | 「没有值」是**正常结果** | 自己决定缺省行为 |
| `Result<T, E>` | 操作**可能失败**，失败原因要保留 | 处理或继续传播 |
| 直接 `panic` | 「这不可能发生」 | 什么都不用做——但猜错就是崩溃 |

这一章的目标只有一个：**让这三个选择变成有意识的决定，而不是习惯**。

---

## 1. `Option<T>`：把「没有」变成类型

```rust,ignore
pub enum Option<T> {
    None,
    Some(T),
}
```

它看起来平凡得不像能解决问题。关键差别在于：**`Option<T>` 与 `T` 是两个不同的类型，编译器不允许你混用。** 在 C、Java、Go 里，`null` 属于每一个引用类型，所以「这个函数可能返回 null」只能写在文档里，靠调用方自觉；在 Rust 里它是签名的一部分，忘了处理就编译不过。

### 它不一定要多占内存

Rust 会为 `Option` 做「空指针优化」（niche optimization）：如果一个类型内部存在不可能出现的位模式，`None` 就可以借用它表示，不额外占空间。

```rust,editable
fn main() {
    println!("&u8            = {}", std::mem::size_of::<&u8>());
    println!("Option<&u8>    = {}", std::mem::size_of::<Option<&u8>>());
    println!("Box<u8>        = {}", std::mem::size_of::<Box<u8>>());
    println!("Option<Box<u8>> = {}", std::mem::size_of::<Option<Box<u8>>>());

    // 但 u16 用满了全部位模式，就没有免费的 None 可用
    println!("u16            = {}", std::mem::size_of::<u16>());
    println!("Option<u16>    = {}", std::mem::size_of::<Option<u16>>());
}
```

这条事实会反复出现在性能讨论里：**`Option<&T>` 和 `&T` 一样大，`Option<u16>` 不是。** 所以「为了省内存而到处传裸指针」在 Rust 里通常没有必要。

### 组合子：把「判空」写进类型变换里

新手写法是先判断再取值，一层套一层：

```rust,editable
struct Target {
    url: String,
    alias: Option<String>,
}

fn label(target: &Target) -> String {
    match &target.alias {
        Some(alias) => format!("{alias} ({})", target.url),
        None => target.url.clone(),
    }
}

fn main() {
    let with_alias = Target { url: String::from("https://example.com"), alias: Some(String::from("Example")) };
    let without = Target { url: String::from("https://example.com"), alias: None };
    println!("{}", label(&with_alias));
    println!("{}", label(&without));
}
```

同一件事用组合子写，意图更直接：

```rust,editable
fn main() {
    let alias: Option<String> = Some(String::from("Example"));
    let empty: Option<String> = None;
    let url = "https://example.com";

    let label = alias
        .as_deref()                       // Option<String> -> Option<&str>，避免移动
        .map(|name| format!("{name} ({url})"))   // 有值时变换
        .unwrap_or_else(|| url.to_owned());      // 没值时给出缺省

    let missing = empty
        .as_deref()
        .map(|name| format!("{name} ({url})"))
        .unwrap_or_else(|| url.to_owned());

    println!("{label}");
    println!("{missing}");
}
```

常用的组合子，按用途分类：

| 组合子 | 作用 | 什么时候用 |
|---|---|---|
| `map` | 有值时把值换成另一个值 | 变换不需要失败 |
| `and_then` | 有值时执行另一个可能返回 `Option` 的操作 | 链式查找（查 A，找到再查 B） |
| `filter` | 不满足条件就当没有 | 校验可选项 |
| `or` / `or_else` | 没有值时用备选值 | 兜底 |
| `unwrap_or` / `unwrap_or_else` / `unwrap_or_default` | 取出值或给缺省 | 缺省值便宜用前者，需要计算用后者 |
| `ok_or` / `ok_or_else` | 把 `Option` 变成 `Result` | 把「没有」升级成「错误」 |
| `take` / `replace` | 从 `&mut Option<T>` 里搬走值 | 状态机、避免 clone |
| `as_ref` / `as_deref` | 借用里面的值而不是移动它 | 在 `&Option<T>` 上链式调用 |

> 判断要不要用组合子，看一句话：**读代码的人能不能一眼看出「主线」和「异常分支」。** 三个以内的 `map`/`and_then` 通常比 `match` 清楚；再长就该拆函数了。

### `unwrap` 什么时候是合理的

```rust,should_panic
fn main() {
    let history: Vec<u16> = Vec::new();
    println!("{}", history.last().unwrap());
}
```

`unwrap` 把「可能没有值」的接口偷偷改成了「没有值就终止」。它合理的场合其实很少，但确实存在：

| 场合 | 为什么可以 | 例子 |
|---|---|---|
| 测试 | 失败就该让测试炸掉 | `#[test] fn ...` 里的 `unwrap()` |
| 已经由不变量保证 | 上面的代码刚判断过，且没有并发修改 | `if map.contains_key(k) { map.get(k).unwrap() }` |
| 原型与一次性脚本 | 你不在乎崩溃 | `fn main()` 里的临时验证 |

**输入、网络、文件、数据库这四类边界永远不属于上面任何一种。** 在项目代码里，`unwrap` 应该稀有到每次出现都值得在评审里被问一句。`expect("说明为什么不可能")` 比 `unwrap()` 好，因为它把「为什么我认为这里安全」写下来了。

---

## 2. `Result<T, E>`：把「失败」变成类型

```rust,ignore
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

和异常相比，差别不在语法糖，而在**控制流的所有权**：

| 维度 | 异常（Java/Python/C#） | `Result` |
|---|---|---|
| 是否可忽略 | 可以（不写 catch 就穿透） | 不能：`#[must_use]` 会警告，`?` 要显式写 |
| 是否在签名里 | 通常不在（checked exception 例外） | 一定在 |
| 是否是值 | 不是，控制流跳转 | 是：可以存进 `Vec`、放进 `struct`、排序、统计 |
| 失败类型 | 通常是异常类层次 | 由你决定的类型 |
| 零成本 | 抛异常要栈展开 | 就是一个枚举，与手写 `match` 同价 |

「错误是值」这条最容易被低估。它意味着你可以把一批失败收集起来一起汇报：

```rust,editable
fn parse_all(values: &[&str]) -> (Vec<u16>, Vec<String>) {
    let mut ok = Vec::new();
    let mut failed = Vec::new();
    for value in values {
        match value.parse::<u16>() {
            Ok(port) => ok.push(port),
            Err(error) => failed.push(format!("{value}: {error}")),
        }
    }
    (ok, failed)
}

fn main() {
    let (ok, failed) = parse_all(&["8080", "http", "443"]);
    println!("成功 {ok:?}");
    println!("失败 {failed:?}");
}
```

用异常的语言写这段代码会别扭得多：要么第一个错误就中断，要么引入一个专门的「聚合异常」类型。
---

## 3. `?` 到底是什么

`?` 不是「忽略错误」，也不是「抛异常」。它是**一个提前返回 + 一次类型转换**的语法糖：

```rust,editable
fn read_port(text: &str) -> Result<u16, std::num::ParseIntError> {
    let port = text.parse::<u16>()?;
    Ok(port)
}

fn main() {
    println!("{:?}", read_port("8080"));
    println!("{:?}", read_port("http"));
}
```

上面那个 `?` 展开后等价于：

```rust,ignore
fn read_port(text: &str) -> Result<u16, std::num::ParseIntError> {
    let port = match text.parse::<u16>() {
        Ok(value) => value,
        Err(error) => return Err(From::from(error)),   // 注意 From::from
    };
    Ok(port)
}
```

那个 `From::from` 是关键：它让 `?` 能把一种错误类型自动转成函数签名里的错误类型。项目里 `TargetError`、`StoreError` 能被同一个 `ApiError` 接住，靠的就是这条。

### `?` 也能用在 `Option` 上

```rust,editable
fn first_char(text: &str) -> Option<char> {
    Some(text.chars().next()?)
}

fn main() {
    println!("{:?}", first_char("abc"));
    println!("{:?}", first_char(""));
}
```

但它**不能跨类型**：返回 `Option` 的函数里不能对 `Result` 用 `?`，反之亦然。编译器会直接告诉你怎么办：

```rust,compile_fail
fn parse_port(text: &str) -> Result<u16, std::num::ParseIntError> {
    text.parse::<u16>()
}

fn port_or_default(text: &str) -> Option<u16> {
    Some(parse_port(text)?)
}

fn main() {
    println!("{:?}", port_or_default("8080"));
}
```

```text
error[E0277]: the `?` operator can only be used on `Option`s, not `Result`s,
              in a function that returns `Option`
  |
5 | fn port_or_default(text: &str) -> Option<u16> {
  | --------------------------------------------- this function returns an `Option`
6 |     Some(parse_port(text)?)
  |                          ^ use `.ok()?` if you want to discard the
  |                            `Result<Infallible, ParseIntError>` error information
```

注意编译器的建议：`.ok()?` 会把 `Err` 直接丢掉（变成 `None`），`.ok_or(...)?` 才是「保留错误」的写法——但那时函数就该返回 `Result` 了。**编译器的建议不一定符合你的意图，它只保证能编译过。**

### `main` 也可以返回 `Result`

```rust,editable
use std::num::ParseIntError;

fn main() -> Result<(), ParseIntError> {
    let port: u16 = "8080".parse()?;
    println!("port = {port}");
    Ok(())
}
```

返回 `Err` 时，进程会打印 `Error: ...` 并以非零状态退出。这适合一次性工具；产品代码仍然应该自己决定退出码和输出格式（见第 5 节）。
---

## 4. 错误类型怎么设计

这是本章唯一需要背的决策表。按**代码所在的位置**选，而不是按个人偏好：

| 位置 | 错误类型 | 理由 |
|---|---|---|
| 库 / 领域层 | 自定义枚举，每个变体一个可匹配的类别 | 调用方需要**按类型分支**，不能解析文案 |
| 变体里带上底层原因 | `#[source]` 或手写 `Error::source` | 保留因果链，便于排障 |
| 应用层（main、CLI、handler） | `Box<dyn Error>` 或 `anyhow::Error` | 只需要一路向上报告，不需要分支 |
| 协议边界（HTTP、CLI 退出码） | 一个把领域错误映射成状态码的 `From` 实现 | 边界是转换点，不是字符串拼接点 |

### 反面教材：字符串当错误类型

项目重构前的 `StoreError` 长这样：

```rust,ignore
pub struct StoreError {
    message: String,     // 里面装的是给人看的句子
}
```

它的代价不是「不优雅」，而是**信息丢失**：

- Web 层想对「目标不存在」返回 `404`，只能去 `contains("not found")` 猜文案；
- 猜错的表现是「本该 404 的请求返回了 500」，而且测试很难发现；
- 文案改一个字，所有靠解析文案的分支同时失效。

改成枚举之后，`match` 就是分支，文档也不必再解释「这句话什么时候出现」：

```rust,ignore
pub enum StoreError {
    NotFound { target_id: i64 },
    TargetMismatch { target_id: i64 },
    Corrupt { detail: String },
    Migration { detail: String },
    Database(sqlx::Error),
}
```

### 保留原因：`Error::source`

错误类型要同时服务两类读者：**程序**需要 `match` 出类别，**人**需要看到完整因果链。后者靠 `source()`：

```rust,editable
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum ConfigError {
    MissingPort,
    BadPort { source: ParseIntError },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPort => write!(formatter, "no port was configured"),
            Self::BadPort { source } => write!(formatter, "port is not a number: {source}"),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BadPort { source } => Some(source),
            Self::MissingPort => None,
        }
    }
}

fn port(text: Option<&str>) -> Result<u16, ConfigError> {
    let text = text.ok_or(ConfigError::MissingPort)?;
    text.parse().map_err(|source| ConfigError::BadPort { source })
}

fn main() {
    for input in [Some("8080"), Some("http"), None] {
        match port(input) {
            Ok(port) => println!("port = {port}"),
            Err(error) => {
                println!("error: {error}");
                if let Some(source) = error.source() {
                    println!("  caused by: {source}");
                }
            }
        }
    }
}
```

三个要点：`ok_or` 把 `Option` 升级成 `Result`；`map_err` 把底层错误包装成领域错误但**保留**它；`source()` 让调用方打印完整链条。生产代码里打印错误链条通常交给 `anyhow` 或 `tracing`，但原理就是这三行。

---

## 5. 回到项目：三层映射

同一个失败在项目里被翻译三次，每一层只做自己那层的决定。

**第一层：领域层判定类别**

```rust,ignore
pub enum TargetError {
    EmptyName,           // 名字是空的
    InvalidUrl,          // URL 解析不出来
    UnsupportedScheme,   // 不是 http/https
}
```

**第二层：边界层翻译成协议**

```rust,ignore
impl From<TargetError> for ApiError {
    fn from(error: TargetError) -> Self {
        // 用户输入的问题 → 400，而不是 500
        Self::bad_request("invalid_target", error.to_string())
    }
}

impl From<StoreError> for ApiError {
    fn from(error: StoreError) -> Self {
        let message = error.to_string();
        match error {
            StoreError::NotFound { .. }       => Self::not_found(message),    // 404
            StoreError::TargetMismatch { .. } => Self::conflict(message),     // 409
            _                                 => Self::internal(message),     // 500
        }
    }
}
```

**第三层：CLI 翻译成退出码**

```text
0  命令跑完了，每个站点都响应了
1  命令跑完了，至少一个站点没响应
2  命令根本没跑起来：参数错、文件读不到、URL 非法
```

### `Result<Option<T>, E>`：两个问题，两个轴

```rust,ignore
fn latest_result(&self, target_id: i64) -> Result<Option<CheckResult>, StoreError>;
```

签名读作：「这次查询**成功**了吗（外层）；如果成功，**有记录**吗（内层）」。三种组合的含义完全不同：

| 返回值 | 含义 | 调用方该做什么 |
|---|---|---|
| `Ok(Some(result))` | 查到了 | 正常使用 |
| `Ok(None)` | 查询成功，但这个目标还没有记录 | 返回 404，或显示「暂无数据」 |
| `Err(e)` | 查询本身失败（数据库坏了） | 返回 500，并记录日志 |

如果把它压成一个布尔值或者一个 `Option`，你就再也分不清「没有数据」和「读不到数据」——前者是正常状态，后者是故障。
---

## 6. 深水区

### 6.1 组合子链该在哪里停下来

组合子是给「主线 + 一两个分支」用的。当一段逻辑出现三层以上的嵌套分支，或者需要在分支之间共享局部变量时，`match` 反而更清楚。一个可操作的判断：

> 如果给这段链式调用写一行注释都说不清它在做什么，就改成 `match` 或者抽成一个函数。

### 6.2 `collect` 能直接收集成 `Result`

这是最被低估的一招：一个元素失败，整批就失败，但**第一个错误会被保留**。

```rust,editable
fn main() {
    let all_valid = ["8080", "443"];
    let with_one_bad = ["8080", "http", "443"];

    let ports: Result<Vec<u16>, _> = all_valid.iter().map(|text| text.parse::<u16>()).collect();
    println!("全部合法: {ports:?}");

    let ports: Result<Vec<u16>, _> = with_one_bad.iter().map(|text| text.parse::<u16>()).collect();
    println!("有一个坏的: {ports:?}");

    // 换成 Option 也可以：任一为 None 则整个为 None
    let firsts: Option<Vec<char>> = ["ab", "cd"].iter().map(|text| text.chars().next()).collect();
    println!("全部有值: {firsts:?}");
}
```

在没有异常的语言里，「批量解析，遇到第一个错误就返回」通常要写一个显式循环加提前返回。`collect` 把它变成了一次类型标注。

### 6.3 `Option<Option<T>>` 是危险的信号

```rust,ignore
fn find_alias(&self, id: i64) -> Option<Option<String>>;   // 别这么写
```

调用方看到 `None` 时无法区分「查不到这个 id」和「查到了，但它没有别名」。这种歧义应该用枚举消除：

```rust,editable
#[derive(Debug)]
enum AliasLookup {
    TargetNotFound,
    NoAlias,
    Alias(String),
}

fn main() {
    let cases = [AliasLookup::TargetNotFound, AliasLookup::NoAlias, AliasLookup::Alias(String::from("Example"))];
    for case in cases {
        match case {
            AliasLookup::TargetNotFound => println!("这个目标不存在"),
            AliasLookup::NoAlias => println!("存在，但没有别名"),
            AliasLookup::Alias(alias) => println!("别名是 {alias}"),
        }
    }
}
```

`Result<Option<T>, E>` 之所以没问题，是因为外层的 `Err` 和 `Option` 的表达力不重叠：一个说「操作失败」，一个说「操作成功但没有值」。

### 6.4 `ok_or` 与 `ok_or_else`

```rust,ignore
value.ok_or(TargetError::InvalidUrl)               // 错误值很便宜：直接构造
value.ok_or_else(|| TargetError::from(raw.clone())) // 需要计算才构造，或者要 clone
```

区别只有一条：`ok_or` 会**无条件**构造参数（即使值存在），`ok_or_else` 只在需要时报错。错误类型是零成本的枚举时无所谓；错误信息要格式化字符串时，`ok_or` 会白白分配内存。

---

## 7. 常见误解

| 误解 | 准确说法 |
|---|---|
| 「`?` 就是 try/catch」 | 它是**提前返回 + `From::from` 转换**。没有栈展开，错误是普通值，可以存进集合。 |
| 「`unwrap()` 只是图省事」 | 它把「可恢复的失败」改成了「必然崩溃」。在输入、网络、文件、数据库边界不可接受。 |
| 「错误类型越细越好」 | 每个变体都要有调用方真的会 `match` 它，否则只是噪音。三个类别通常够用。 |
| 「`Box<dyn Error>` 到处都能用」 | 库的公开 API 用它，调用方就无法按类型分支；它是应用层的工具。 |
| 「`Option` 一定零成本」 | 只在有空位（niche）时成立。`Option<&T>` 是，`Option<u16>` 不是。 |
| 「`Result` 比异常慢」 | 它就是一个枚举加一次 `match`，与手写分支同价；异常才有栈展开的开销。 |

---

## 8. 练习与自测

### 练习

```sh
cargo test -p monitor-domain
cd exercises
rustlings run 04_result
```

完成 `04_result` 后，给自己解释：**为什么不能把 `Result<Option<T>, E>` 简化成一个布尔值？**

### 自测清单（能全部做到才算掌握）

1. 说出 `Option` 与 `Result` 的语义边界，并各举一个项目里的真实函数。
2. 把 `?` 展开成 `match`，并说明 `From::from` 在其中做什么。
3. 解释为什么 `Result<Option<T>, E>` 不能压成 `Option<Result<T, E>>`。
4. 各举一个 `unwrap` 合理与不合理的场合，并说明判断依据。
5. 手写一个带 `source()` 的错误枚举，并让 `main` 打印出完整因果链。
6. 用一次 `collect` 把 `Vec<&str>` 转成 `Result<Vec<u16>, _>`。
7. 说明 `ok_or` 与 `ok_or_else` 的选择依据。

第 5、6 题写完以后，回头看一眼 `monitor-domain` 里 `TargetError` 的定义——你应该能说出它为什么是三个变体，而不是一个 `String`。

---

## AI 辅导提示词

这三段可以直接复制给 AI 助手（Kimi、ChatGPT 等）。它们的设计意图是**让助手出题和追问，而不是替你写代码**——完整方法论见[用 AI 助手当教练](../guided/ai-tutor.md)。

```text
请出 5 道判断题，考察 Option 与 Result 的边界情况，例如：
「这段代码会 panic 吗」「这个 ? 能编译吗」「这个签名能不能简化」。
先只出题。我答完后再逐条点评，并指出我理由里的错误。
```

```text
下面是我的函数签名和它可能失败的方式。请帮我决定错误类型的形状：
- 该用枚举、还是 Box<dyn Error>、还是直接复用标准库错误？
- 每个变体会不会有调用方真的去 match 它？
请先问我三个问题再给结论，不要直接给代码。

[粘贴函数签名和失败场景]
```

```text
请把下面这段代码里的 ? 运算符全部展开成等价的 match 形式
（包括 From::from 的那一步），然后指出：
哪些地方的自动转换是我依赖的，哪些地方其实发生了信息丢失。

[粘贴代码]
```
