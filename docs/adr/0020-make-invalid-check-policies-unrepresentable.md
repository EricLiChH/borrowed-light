# 让非法的检查策略无法被构造

`CheckPolicy::new` 原本接收四个位置参数并返回 `Result<Self, PolicyError>`，用来拒绝 `max_attempts == 0` 和 `concurrency == 0`。两个参数是含义相反的 `Duration`，调用点写成 `CheckPolicy::new(5s, 2, 100ms, 8)` 很难看出哪个是超时、哪个是退避。

现在 `max_attempts` 与 `concurrency` 是 `NonZeroUsize`，零值在类型层面就不存在，`PolicyError` 随之删除，构造不再可能失败。其余字段由 `CheckPolicy::builder()` 具名设置，默认值就是 CLI 与 Web 使用的生产默认值。

退避也从“一个固定的 `Duration`”变成 `Backoff` 值对象：`none`、`fixed`、`exponential`，可设置上限与抖动。`Backoff::delay(retry, fraction)` 是纯函数——随机数是参数而不是隐藏状态——所以退避曲线可以脱离时钟和网络被单元测试。
