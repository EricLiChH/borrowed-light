# 入门诊断

> **怎么用**：先预测，再运行。每题只检查一个信号：**你能否预测 Rust 的行为**，而不是是否记得术语定义。
>
> 记录你的状态用三档，而不是对错：**有把握 / 犹豫 / 完全不确定**。「有把握但答错」比「不确定」更值得注意——那说明你有一个稳固但错误的模型。

---

## 第一部分：语法层（能靠查阅解决，不必重学）

### 1. 哪个变量可以修改？

```rust,editable
fn main() {
    let attempts = 1;
    let mut successes = 0;
    successes += attempts;
    println!("{successes}");
}
```

<details><summary>查看答案</summary>

只有 `successes` 可以重新赋值；输出是 `1`。

如果你犹豫了，把「变量默认不可变」加入快速复习清单。

</details>

### 2. 遮蔽之后，原来的绑定还在吗？

```rust,editable
fn main() {
    let value = 1;
    let value = value + 1;     // 新的绑定，还是修改旧的？
    println!("{value}");
}
```

<details><summary>查看答案</summary>

输出 `2`。`let value = ...` 第二次出现时**创建了一个新绑定**，右侧的 `value` 仍然指旧的。这和 `let mut value` 完全不同：遮蔽可以换类型，`mut` 不行。

</details>

### 3. `?` 能用在 `main` 里吗？

```rust,editable
fn main() -> Result<(), std::num::ParseIntError> {
    let port: u16 = "8080".parse()?;
    println!("{port}");
    Ok(())
}
```

<details><summary>查看答案</summary>

可以。`main` 的返回类型实现了 `Termination`，所以 `Result` 是合法的返回类型；返回 `Err` 时进程打印错误并以非零状态退出。

</details>

---

## 第二部分：所有权层（这一层错了，后面全塌）

### 4. 下面能编译吗？

```rust,compile_fail
fn main() {
    let url = String::from("https://example.com");
    let queued = url;
    println!("{url} -> {queued}");
}
```

<details><summary>查看答案</summary>

不能。`String` 被移动到 `queued`，之后不能再读取 `url`（`E0382`）。这是[所有权章节](ownership.md)的第一个实验。

</details>

### 5. `len` 会拿走字符串吗？

```rust,editable
fn len(text: &str) -> usize {
    text.len()
}

fn main() {
    let url = String::from("https://example.com");
    println!("{} {url}", len(&url));
}
```

<details><summary>查看答案</summary>

不会。`len` 只借用 `url`，借用结束后原值仍然可用。**参数是 `&T` 还是 `T`，决定了调用方是否还能用它。**

</details>

### 6. 为什么这段代码冲突？

```rust,compile_fail
fn main() {
    let mut urls = vec![String::from("https://example.com")];
    let first = &urls[0];
    urls.push(String::from("https://rust-lang.org"));
    println!("{first}");
}
```

<details><summary>查看答案</summary>

`first` 是对 `urls` 内部数据的不可变借用；`push` 需要可变借用，而且可能重新分配内存（那样 `first` 就悬空了）。在 `first` 最后一次使用之前，两者不能重叠（`E0502`）。

**追问**：删掉哪一行它就能编译？——删掉最后的 `println!("{first}")`。这说明冲突的判据是**借用的活跃区间**，不是花括号范围。

</details>

### 7. 两个可变借用能同时存在吗？

```rust,compile_fail
fn main() {
    let mut url = String::from("example.com");
    let first = &mut url;
    let second = &mut url;
    first.push('!');
    second.push('?');
    println!("{first} {second}");
}
```

<details><summary>查看答案</summary>

不能（`E0499`）。`&mut T` 是**独占**权限，同一时间只能有一个。注意冲突发生在 `first` 还要被使用的前提下：如果 `first` 在 `second` 创建之前就不再使用，代码就能编译。

</details>

### 8. 部分移动之后，结构体还能用吗？

```rust,compile_fail
#[derive(Debug)]
struct Target {
    name: String,
    url: String,
}

fn main() {
    let target = Target {
        name: String::from("Rust"),
        url: String::from("https://www.rust-lang.org"),
    };
    let url = target.url;          // 只把 url 移走
    println!("moved {url}");
    println!("{target:?}");        // 这一行呢？
}
```

<details><summary>查看答案</summary>

不能整体使用（`E0382`：borrow of partially moved value），但**剩下的字段仍然可以单独用**（`target.name` 是好的）。所有权是按字段算的。

</details>
---

## 第三部分：类型系统层（决定你写出的代码长什么样）

### 9. 这两种建模，哪种更难写错？

```text
// 甲
struct LooseResult {
    reachable: bool,
    status: Option<u16>,
    reason: Option<String>,
}

// 乙
enum CheckOutcome {
    Reachable { status: u16 },
    Unreachable { kind: CheckFailureKind, reason: String },
}
```

<details><summary>查看答案</summary>

乙。甲的三个字段可以组合出无意义的状态（`reachable: true` 但 `status: None`），只能靠约定避免；乙根本表达不出矛盾状态。

这是本课程反复出现的原则：**让非法状态无法表示**。它在项目里出现过三次：`CheckOutcome`、`StoreError`、`ApiError`。

</details>

### 10. 这个函数返回的是什么类型？

```rust,editable
fn numbers() -> impl Iterator<Item = u16> {
    [200_u16, 404].into_iter().filter(|status| *status < 300)
}

fn main() {
    println!("{}", numbers().count());
}
```

<details><summary>查看答案</summary>

编译期确定的某个具体类型，只是**对调用方隐藏**——`impl Trait` 不是 `dyn`，没有装箱、没有虚表。

如果这题你有把握，回答追问：什么情况下必须改用 `dyn Trait`？——当运行期需要在同一个变量或集合里放**不同的实现**时。注意 `async fn` 写在 trait 里时，它的 future 不满足 `Send`，那会让 `dyn` 方案直接不可用（决策记录见仓库的 `docs/adr/0017`）。

</details>

### 11. `?` 能同时用于 `Option` 和 `Result` 吗？

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

<details><summary>查看答案</summary>

不能跨类型使用（`E0277`）。返回 `Option` 的函数里只能对 `Option` 用 `?`。编译器会建议 `.ok()?`，但那会**丢掉错误信息**；真正该问的是「这个函数到底该返回 `Option` 还是 `Result`」。

</details>

---

## 第四部分：工具链（忘掉命令不算问题，但要能重建）

### 12. 这四个命令各自做什么？

```text
cargo check
cargo test
cargo clippy
cargo fmt
```

<details><summary>查看答案</summary>

| 命令 | 作用 | 什么时候用 |
|---|---|---|
| `cargo check` | 只做类型检查，不生成可执行文件 | 改代码时最快的内循环 |
| `cargo test` | 编译并运行测试（含文档测试） | 每次改完逻辑 |
| `cargo clippy` | 静态检查地道写法 | 提交前；配合 `-- -D warnings` |
| `cargo fmt` | 按官方风格格式化 | 写完就格式化，避免评审纠结排版 |

忘记命令不是问题；**项目检查点要求它们都通过**，所以要重建肌肉记忆。

</details>

### 13. 为什么本项目的测试不访问公网？

<details><summary>查看答案</summary>

三个原因：**可重复**（公网站点会变、会限流）、**快**（本地 TCP 往返是微秒级）、**不受环境影响**（公司代理会改变请求的去向，甚至让不可达的地址报告 200）。

项目用 `127.0.0.1:0` 上的临时服务器替代公网，见 `docs/adr/0011`。

</details>

---

## 跳过测试：给「我觉得我还会」的人

这四题不需要新知识，但很容易答错。**任何一题不确定，就不该跳过对应章节。**

### 14. 创建 future 会发生什么？

```rust,editable
fn main() {
    let future = async {
        println!("future 内部执行了");
        42
    };

    println!("future 已经创建");
    drop(future);
}
```

<details><summary>查看答案</summary>

只打印「future 已经创建」。**创建 future 不做任何事**，只有被 poll 才前进。这是异步的第一条心智模型，见 [Future、任务与运行时](../async/future-runtime.md)。

</details>

### 15. 它的借用在哪里结束？

```rust,editable
fn main() {
    let mut statuses = vec![200_u16, 404, 503];
    let first = &statuses[0];
    println!("first = {first}");     // 最后一次使用
    statuses.push(500);              // 这里还冲突吗？
    println!("{statuses:?}");
}
```

<details><summary>查看答案</summary>

不冲突。借用在**最后一次使用处**结束（non-lexical lifetimes），不是作用域结束时。把 `println!` 挪到 `push` 之后就恢复冲突。

</details>

### 16. 它能跨线程共享吗？

```text
use std::cell::RefCell;
use std::rc::Rc;

let shared = Rc::new(RefCell::new(Vec::<u16>::new()));
// 交给 thread::spawn 会怎样？
```

<details><summary>查看答案</summary>

会被拒绝（`E0277`：`Rc<RefCell<Vec<i32>>>` cannot be sent between threads safely）。**两个原因各自都足以拒绝**：`Rc` 的引用计数不是原子的，`RefCell` 的运行期借用表也不是线程安全的。

替代：`Rc → Arc`、`RefCell → Mutex`。见[线程章节](../async/threads-channels-state.md)。

</details>

---

## 生成你的路线

按**不确定的题数**（不是答错的题数）选择起点：

| 不确定题数 | 起点 | 建议投入 | 说明 |
|---|---|---|---|
| 10 题以上 | 从[所有权](ownership.md)开始顺序读 | 30–40 小时 | 走完整标准路径，不要跳阶段门 |
| 6–9 题 | 跳过第一部分，从[值、枚举与集合](../core/values-collections-modules.md)开始 | 20–25 小时 | 语法层靠查阅补，不要重学 |
| 3–5 题 | 直接做练习，卡住再回查对应章节 | 12–15 小时 | 优先跑 `rustlings` 与项目测试 |
| 0–2 题 | 直接进 [CLI 阶段](../cli/monitor-cli.md) | 8–10 小时 | 把时间花在工程与异步上 |

**分层的现实意义**：语法层忘得最快也最容易补；所有权/类型系统层忘得慢，但一旦没建立过，后面每一步都会卡；工具链层靠用一次就能重建。

### 什么**不算**诊断信号

- 忘了某个 API 的名字（查文档即可，不是能力问题）
- 记不住错误码（编译器会告诉你，读它就行）
- 没写过 `async`、没用过 `SQLx`（那是后面要学的内容，不是"退步"）

### 下一步

[所有权：移动、借用与可变借用](ownership.md)。如果诊断结果指向跳过，直接去[30–40 小时标准路径](standard-path.md)选择阶段门。
