# 入门诊断

先答，再运行。每题只检查一个信号：你能否预测 Rust 的行为，而不是是否记得术语定义。

## 1. 哪个变量可以修改？

```rust
fn main() {
    let attempts = 1;
    let mut successes = 0;
    successes += attempts;
    println!("{successes}");
}
```

<details><summary>查看答案</summary>

只有 `successes` 可以重新赋值；输出是 `1`。如果你犹豫了，把“变量默认不可变”加入快速复习清单。

</details>

## 2. 下面能编译吗？

```rust,compile_fail
fn main() {
    let url = String::from("https://example.com");
    let queued = url;
    println!("{url} -> {queued}");
}
```

<details><summary>查看答案</summary>

不能。`String` 被移动到 `queued`，之后不能再读取 `url`。这是本章第一个实验。

</details>

## 3. `len` 会拿走字符串吗？

```rust
fn len(text: &str) -> usize {
    text.len()
}

fn main() {
    let url = String::from("https://example.com");
    println!("{} {url}", len(&url));
}
```

<details><summary>查看答案</summary>

不会。`len` 只借用 `url`，借用结束后原值仍然可用。

</details>

## 4. 为什么这段代码冲突？

```rust,compile_fail
fn main() {
    let mut urls = vec![String::from("https://example.com")];
    let first = &urls[0];
    urls.push(String::from("https://rust-lang.org"));
    println!("{first}");
}
```

<details><summary>查看答案</summary>

`first` 是对 `urls` 内部数据的不可变借用；`push` 需要可变借用，而且可能重新分配内存。在 `first` 最后一次使用之前，两者不能重叠。

</details>

## 5. `cargo check` 与 `cargo test` 的区别是什么？

<details><summary>查看答案</summary>

`cargo check` 快速检查代码能否通过编译，不生成最终程序；`cargo test` 还会构建并运行测试。忘记命令不是问题，但应在项目检查点重新建立肌肉记忆。

</details>

## 6. 你会选择哪个参数？

需求：函数只读取 `MonitorTarget`，调用后调用者还要继续使用它。

- `target: MonitorTarget`
- `target: &MonitorTarget`
- `target: &mut MonitorTarget`

<details><summary>查看答案</summary>

选择 `&MonitorTarget`。它表达只读借用，既不接管所有权，也不承诺修改。

</details>

## 生成你的路线

- 0–1 题不确定：完整学习本章并完成全部练习。
- 2–3 题不确定：先读每节的“状态图”，再直接做练习。
- 4–6 题全部确定：可以先做 Rustlings；若一次通过，直接进入项目检查点。

下一步：[所有权：移动、借用与可变借用](ownership.md)。

