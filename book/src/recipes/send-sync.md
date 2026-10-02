# Send / Sync：异步任务为什么拒绝这个值

| 元数据 | 内容 |
|---|---|
| 任务 | 判断一个值能否跨线程，并改成能跨线程的形状 |
| 概念 | `Send`、`Sync`、`Arc`/`Mutex`、闭包捕获 |
| 错误码 | `E0277` |
| 先修 | 所有权、`Rc`/`RefCell` |
| 项目阶段 | 并发检查器、Web 共享状态 |

## 现象

```rust,compile_fail
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;

fn main() {
    let statuses = Rc::new(RefCell::new(Vec::new()));
    let shared = Rc::clone(&statuses);
    thread::spawn(move || shared.borrow_mut().push(200)).join().unwrap();
    println!("{:?}", statuses.borrow());
}
```

```text
error[E0277]: `Rc<RefCell<Vec<i32>>>` cannot be sent between threads safely
   |
 8 |     thread::spawn(move || shared.borrow_mut().push(200)).join().unwrap();
   |     ------------- -------^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |     |             |
   |     |             `Rc<RefCell<Vec<i32>>>` cannot be sent between threads safely
   |     |             within this closure
   |     |
   |     required by a bound introduced by this call
   |
   = help: the trait `Send` is not implemented for `Rc<RefCell<Vec<i32>>>`
```

## 判断

```text
Send : 值的「所有权」可以安全地移到另一个线程
Sync : 「&T」可以安全地被多个线程共享
```

两者都是**自动 trait**：由字段自动推导，不能手写实现（写了也是 `unsafe`）。

| 类型 | `Send` | `Sync` | 原因 |
|---|---|---|---|
| `i32`、`String`、`Vec<T>` | ✅ | ✅ | 拥有自己的数据 |
| `&T`（`T: Sync`） | ✅ | ✅ | 共享引用只读 |
| `&mut T`（`T: Send`） | ✅ | ❌ | 独占引用只有一个使用者 |
| `Rc<T>` | ❌ | ❌ | 引用计数不是原子的 |
| `RefCell<T>` | ✅ | ❌ | 运行期借用表没有跨线程同步 |
| `Arc<T>` | ✅* | ✅* | 原子计数；取决于 `T` |
| `Mutex<T>` | ✅* | ✅* | 锁即同步手段；取决于 `T` |

上面那个例子**两个原因各自都足以拒绝它**。

## 修复选择

| 想要 | 单线程 | 多线程 |
|---|---|---|
| 共享所有权 | `Rc<T>` | `Arc<T>` |
| 内部可变性 | `RefCell<T>` | `Mutex<T>` / `RwLock<T>` |
| 传递状态变化 | — | `mpsc` / `tokio::sync::mpsc` |

**不要用 `unsafe` 绕开。** 编译器拒绝的不是形式，是真实的数据竞争。

## 异步里的同一件事

`tokio::spawn` 的约束是 `Send + 'static`：

- `Send`：任务可能先在 A 线程被 poll，之后被迁到 B 线程；
- `'static`：任务的生命周期由运行时管理，不能借用外部栈数据。

所以异步代码里到处是 `move`——不是为了性能，是为了满足 `'static`。

深入阅读：[线程、消息与共享状态](../async/threads-channels-state.md#5-send-与-sync两个精确的定义)。
