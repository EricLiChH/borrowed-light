# E0382：值已被移动

## 何时出现

一个非 `Copy` 值已经移动到新位置，旧绑定又被使用。

```rust,compile_fail
fn main() {
    let url = String::from("https://example.com");
    let queued = url;
    println!("{url} {queued}");
}
```

## 先判断意图

1. 新位置应该长期拥有值吗？接受 move，停止使用旧绑定。
2. 函数只需要读取吗？把接口改成 `&T`。
3. 两边确实都要独立拥有吗？最后才考虑 `.clone()`。

```rust
fn display(url: &str) {
    println!("{url}");
}

fn main() {
    let url = String::from("https://example.com");
    display(&url);
    println!("still owned here: {url}");
}
```

返回[所有权样章](../guided/ownership.md#实验一move-把责任一起交出去)。

