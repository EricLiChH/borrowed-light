# 项目检查点：第一批领域类型

网站健康监测器现在有三个核心概念：

- `MonitorTarget` 拥有名称和 URL，并守住最小输入不变量。
- `CheckResult` 拥有一次检查的快照；创建时只借用目标。
- `CheckHistory` 通过 `record(&mut self, result: CheckResult)` 接管结果，通过 `latest(&self)` 借出最近结果。

运行检查点：

```sh
cargo test -p monitor-domain
```

下面不是伪代码，而是 workspace 中实际运行的行为测试：

```rust,ignore
{{#include ../../../projects/monitor-domain/tests/check_history.rs}}
```

观察接口中的三个所有权选择：

```rust,ignore
let result = CheckResult::reachable(&target, 200); // 借用目标
history.record(result);                            // 移动结果
let latest = history.latest();                    // 借用历史
```

## 迁移题

不要运行，先回答：如果 `latest` 的返回类型从 `Option<&CheckResult>` 改成 `Option<CheckResult>`，调用者和实现各要付出什么代价？

<details><summary>参考答案</summary>

返回拥有的 `CheckResult` 会尝试把元素移出 `Vec`；实现必须删除该元素、clone，或改变存储结构。调用者得到所有权，但“只查看最新结果”这个普通操作会变得昂贵或破坏历史记录。借用返回更符合接口意图。

</details>

完成标准：你能不用“因为编译器规定”这句话，解释测试里三处 move/borrow 的原因。

