# 错误按类型区分，而不是装进字符串

`StoreError` 原本是 `struct StoreError { message: String }`。调用方只能靠解析文案来判断发生了什么，于是“目标不存在”和“数据库坏了”在 HTTP 层都变成 `500`，`save_result` 的“找不到目标”也只能打印一行字。

现在它是枚举：`NotFound { target_id }`、`TargetMismatch { target_id }`、`Corrupt { detail }`、`Migration { detail }`、`Database(sqlx::Error)`。Web 层匹配变体决定状态码——`404`、`409`、`500`——而不是猜；`Database` 通过 `Error::source` 保留底层原因。`TargetMismatch` 的文案仍然包含 `does not belong`，因为那是学习者在这一课里要认出的信号。

结论：字符串错误适合原型，一旦有第二个调用方需要按失败类型做决定，就该换成枚举。
