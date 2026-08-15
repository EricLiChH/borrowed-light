# E0502：借用发生冲突

## 何时出现

一个值仍被不可变借用时，又尝试创建可变借用。

```rust,compile_fail
fn main() {
    let mut url = String::from("example.com");
    let view = &url;
    url.insert_str(0, "https://");
    println!("{view}");
}
```

## 优先修复顺序

1. 把只读操作放在修改之前，让不可变借用更早结束。
2. 缩小引用所在的作用域。
3. 重新设计接口，使读取和修改不需要重叠。
4. 只有确实需要独立快照时才 clone。

```rust
fn main() {
    let mut url = String::from("example.com");
    println!("before: {url}");
    url.insert_str(0, "https://");
    println!("after: {url}");
}
```

返回[所有权样章](../guided/ownership.md#实验三mut-t-是临时独占写权限)。

