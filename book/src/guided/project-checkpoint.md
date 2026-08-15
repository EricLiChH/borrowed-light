# 项目检查点：第一批领域类型

网站健康监测器现在有三个核心概念：

- `MonitorTarget` 拥有名称和 URL，并守住最小输入不变量。
- `CheckResult` 拥有一次检查的快照；创建时只借用目标。
- `CheckHistory` 通过 `record(&mut self, result: CheckResult)` 接管结果，通过 `latest(&self)` 借出最近结果。

先运行已经通过的领域模型回归测试：

```sh
cargo test -p monitor-domain
```

## 红灯：亲手接上三个所有权动作

打开 `projects/monitor-domain/tests/learner_checkpoint.rs`：

1. 先预测 `CheckResult` 在哪一行被移动，哪两处只是借用。
2. 只删除测试上的 `#[ignore]`，运行下面的命令，确认测试先失败。

   ```sh
   cargo test -p monitor-domain --test learner_checkpoint
   ```

3. 实现文件里的 `record_then_borrow_latest`，不要 clone，也不要改变领域模型的公开接口。
4. 再运行同一命令，直到测试变绿。

<details><summary>提示 1：方向</summary>

`result` 应该进入历史记录；返回值只需要临时查看历史记录。

</details>

<details><summary>提示 2：关键 API</summary>

先调用 `history.record(result)`，再调用 `history.latest()`。前者接收值，后者返回引用。

</details>

观察接口中的三个所有权选择：

```rust,ignore
let result = CheckResult::reachable(&target, 200); // 借用目标
history.record(result);                            // 移动结果
let latest = history.latest();                    // 借用历史
```

<details><summary>完成后再看参考解</summary>

函数体的惯用解只有两个动作：

```rust,ignore
history.record(result);
history.latest()
```

同一行为也由仓库的常规行为测试持续验证：

```rust,ignore
{{#include ../../../projects/monitor-domain/tests/check_history.rs}}
```

</details>

## 迁移题

不要运行，先回答：如果 `latest` 的返回类型从 `Option<&CheckResult>` 改成 `Option<CheckResult>`，调用者和实现各要付出什么代价？

<details><summary>参考答案</summary>

返回拥有的 `CheckResult` 会尝试把元素移出 `Vec`；实现必须删除该元素、clone，或改变存储结构。调用者得到所有权，但“只查看最新结果”这个普通操作会变得昂贵或破坏历史记录。借用返回更符合接口意图。

</details>

完成标准：你能不用“因为编译器规定”这句话，解释测试里三处 move/borrow 的原因。
