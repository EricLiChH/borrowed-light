# 用 `Option` 与 `Result` 表达分支

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 去掉正常业务分支中的 panic | `Option`、`Result`、`?`、模式匹配 | 90 分钟 | 可验证的目标输入与错误分类 |

先预测：第二次查找没有结果时，函数应该返回空值、错误，还是直接终止程序？

```rust
fn latest_status(history: &[u16]) -> Option<u16> {
    history.last().copied()
}

fn main() {
    println!("{:?}", latest_status(&[200, 204]));
    println!("{:?}", latest_status(&[]));
}
```

- `Option<T>` 表达“没有值也属于正常情况”。
- `Result<T, E>` 表达“操作可能失败，调用者需要决定怎么处理”。
- `?` 不是忽略错误，而是把当前错误沿函数接口返回。

## 故意弄坏

```rust,should_panic
fn main() {
    let history: Vec<u16> = Vec::new();
    println!("{}", history.last().unwrap());
}
```

`unwrap` 把“可能没有值”的接口偷偷改成了“没有值就终止”。在测试、一次性脚本和已经由不变量保证的地方它可能合理；在输入、网络、文件和数据库边界通常不是。

## 迁移到项目

`MonitorRepository::latest_result` 返回 `Result<Option<CheckResult>, StoreError>`：外层 `Result` 表示存储操作可能失败，内层 `Option` 表示存储正常但还没有检查记录。

```sh
cargo test -p monitor-store
```

完成 `04_result` 后，给自己解释：为什么不能把 `Result<Option<T>, E>` 简化成一个布尔值？
