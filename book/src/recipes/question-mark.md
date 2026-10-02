# `?`：传播错误而不丢上下文

| 元数据 | 内容 |
|---|---|
| 任务 | 让错误沿调用链向上走，同时保留原因 |
| 概念 | `?` 的脱糖、`From` 转换、错误类型设计 |
| 错误码 | `E0277`（跨类型使用） |
| 先修 | `Result`、`Option`、`From` |
| 项目阶段 | 领域错误 → HTTP 状态码 |

## 现象一：`?` 不能跨类型

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

编译器的建议（`.ok()?`）能编译，但会**丢掉错误信息**。真正该问的是：这个函数到底该返回 `Option` 还是 `Result`？

## 现象二：`?` 到底做了什么

它等价于「提前返回 + 一次转换」：

```rust,ignore
// 你写的
let port = text.parse::<u16>()?;

// 展开后
let port = match text.parse::<u16>() {
    Ok(value) => value,
    Err(error) => return Err(From::from(error)),   // 注意 From::from
};
```

`From::from` 是关键：它让 `?` 能把一种错误自动转成函数签名里的错误类型。项目里 `TargetError` 和 `StoreError` 能被同一个 `ApiError` 接住，靠的就是这条。

## 判断

| 你想表达 | 用什么 |
|---|---|
| 失败原因对调用方有用 | `Result` + 自定义错误枚举 |
| 「没有值」是正常情况 | `Option` |
| 只关心有没有，不关心为什么 | `.ok()?`（但要意识到信息被丢弃） |
| 底层错误要保留 | `map_err` 包装，并实现 `source()` |

## 修复选择

```rust,ignore
// 1) 升级成 Result：把「没有」变成「错误」
let port = text.parse::<u16>().map_err(ConfigError::BadPort)?;

// 2) 保留原因：包装而不是替换
text.parse().map_err(|source| ConfigError::BadPort { source })

// 3) Option 里确实要吞掉失败：明确写出来
let port = text.parse::<u16>().ok()?;      // 失败与「没有」在这里是同一件事
```

## 项目里的位置

```rust,ignore
impl From<TargetError> for ApiError {
    fn from(error: TargetError) -> Self {
        Self::bad_request("invalid_target", error.to_string())
    }
}
```

有了这个实现，handler 里可以直接写 `MonitorTarget::new(input.name, input.url)?`，错误自动变成 `400`。

深入阅读：[用 Option 与 Result 表达分支](../core/result-option.md#3-到底是什么)。
