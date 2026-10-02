# E0382：值已被移动

| 元数据 | 内容 |
|---|---|
| 任务 | 判断该「借用」「交出」还是「克隆」 |
| 概念 | move、`Copy`、部分移动 |
| 错误码 | `E0382` |
| 先修 | `String` 与 `&str`、函数参数形状 |
| 项目阶段 | 领域模型、检查结果快照 |

## 现象

```rust,compile_fail
#[derive(Debug)]
struct MonitorTarget { url: String }

fn enqueue(target: MonitorTarget) -> Vec<MonitorTarget> {
    vec![target]
}

fn main() {
    let target = MonitorTarget { url: String::from("https://example.com") };
    let queue = enqueue(target);
    println!("{} {}", target.url, queue.len());
}
```

```text
error[E0382]: borrow of moved value: `target`
   |
 9 |     let target = MonitorTarget { url: String::from("https://example.com") };
   |         ------ move occurs because `target` has type `MonitorTarget`,
   |                which does not implement the `Copy` trait
10 |     let queue = enqueue(target);
   |                         ------ value moved here
11 |     println!("{} {}", target.url, queue.len());
   |                       ^^^^^^^^^^ value borrowed here after move
   |
note: consider changing this parameter type in function `enqueue` to borrow
      instead if owning the value isn't necessary
```

## 判断

一句话：**函数真的需要拥有这个值吗？**

| 情形 | 判据 | 修法 |
|---|---|---|
| 只读取 | 函数体里没有需要 `'static` 的存储、没有转移所有权 | 改成 `&T` |
| 确实要接管 | 值会被存进结构体、被返回、或被消耗 | 调用后不要再使用旧绑定 |
| 两边都要 | 调用方后续还要用完整的一份 | `clone()`（能解释理由才用） |

编译器给了两条 note：一条建议改参数为借用，一条提醒可以实现 `Clone`。**它们是方向相反的建议**——选哪条取决于意图，而不是哪条能编译。

## 修复选择

```rust,ignore
// 1) 借：函数只读取，调用方保留所有权
fn enqueue(queue: &mut Vec<MonitorTarget>, target: &MonitorTarget) { queue.push(target.clone()); }

// 2) 交：函数接管，调用方不再使用旧绑定
let queue = enqueue(target);          // target 之后不再出现

// 3) 克隆：两边都真的需要各有一份
let queue = enqueue(target.clone());  // 一次堆分配 + 一次字节复制
```

**第三种最容易被滥用**：它能让程序编译通过，但把一次所有权设计失误变成了一笔运行期开销，而且在评审里看不出意图。

## 项目里的位置

```rust,ignore
// 只借用：同一个目标要反复检查，结果只是读它
pub fn reachable(target: &MonitorTarget, status: u16) -> CheckResult;
fn save_result(&self, target_id: i64, result: &CheckResult) -> Result<(), StoreError>;
```

反例是 `CheckResult` 自己也曾经存了两个 `String` 副本——那是「克隆」思路延伸到类型设计里的结果，见 `docs/adr/0019`。

深入阅读：[所有权：移动、借用与可变借用](../guided/ownership.md#3-e0382移动之后怎么办)。
