# 线程、消息与共享状态

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 比较三种并发所有权形状 | thread、move closure、mpsc、Arc、Mutex、Send、Sync | 3 小时 | 并发地基 |

> **预计 3 小时**。这一章用标准库线程把并发里的所有权问题看清楚，再进入 async。理由和上一章一样：**线程版的报错更直白**，而 `Send`/`Sync` 这套约束在异步里是原样继承的。

---

## 1. 为什么先学线程

异步章节里出现过两条约束：`tokio::spawn` 要求 `Send + 'static`。它们不是 Tokio 发明的，也不是异步特有的——**它们来自「值可能被另一个线程使用」这件事实**。

线程是理解这件事最便宜的工具：`std::thread` 没有运行时、没有额外依赖，报错信息也更直接。

---

## 2. `thread::spawn`：闭包可能活得比函数久

```rust,editable
use std::thread;

fn main() {
    let target = String::from("https://example.com");
    let worker = thread::spawn(move || target.len());
    assert_eq!(worker.join().unwrap(), 19);
}
```

`move` 在这里不是优化，而是**必要条件**。去掉它试试：

```rust,compile_fail
use std::thread;

fn main() {
    let target = String::from("https://example.com");
    let worker = thread::spawn(|| target.len());
    println!("{}", worker.join().unwrap());
}
```

```text
error[E0373]: closure may outlive the current function, but it borrows `target`,
              which is owned by the current function
  |
5 |     let worker = thread::spawn(|| target.len());
  |                                ^^ ------ `target` is borrowed here
  |                                |
  |                                may outlive borrowed value `target`
  |
note: function requires argument type to outlive `'static`
help: to force the closure to take ownership of `target` (and any other
      referenced variables), use the `move` keyword
```

编译器把原因说得很清楚：新线程可能在 `main` 返回之后才运行，那时 `target` 已经不存在了。`move` 把所有权交给闭包，问题消失。

### `join` 不只是「等它跑完」

```rust,editable
use std::thread;

fn main() {
    let worker = thread::spawn(|| panic!("子线程里出了问题"));
    let outcome = worker.join();

    // join 返回 Result：子线程 panic 时，panic 不会静默消失
    println!("子线程是否正常结束 = {}", outcome.is_ok());
}
```

`join()` 返回 `Result<T, Box<dyn Any + Send>>`：成功拿到返回值，失败拿到 panic 载荷。**子线程 panic 不会自动传播到主线程**，但也不会被吞掉——你如果不 `join`，它才真的消失。

### `thread::scope`：不需要 `Arc` 的并发

`move` 与 `'static` 的组合有时很别扭：只是想让几个线程并行读一段数据，却要把所有权搬来搬去。作用域线程解决这个问题：

```rust,editable
use std::thread;

fn main() {
    let target = String::from("https://example.com");

    let length = thread::scope(|scope| {
        // 这里可以借用 target：scope 保证所有线程在离开块之前 join
        let worker = scope.spawn(|| target.len());
        worker.join().unwrap()
    });

    println!("{length}");
    println!("target 仍然可用: {target}");
}
```

`scope` 的承诺是「这个块结束前，里面启动的线程一定都结束了」，所以借用是安全的，不需要 `Arc`，也不需要 `'static`。**当你发现自己在为了并发而到处 clone 字符串，先想想是不是该用 `scope`。**

---

## 3. 消息传递：移动值，不共享容器

```rust,editable
use std::{sync::mpsc, thread};

fn main() {
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || sender.send(204_u16).unwrap());

    assert_eq!(receiver.recv().unwrap(), 204);
}
```

发送后值的所有权进入 channel。接收者不需要锁住发送者的 `Vec`，也不会同时修改同一位置——**「不要通过共享内存来通信，而要通过通信来共享内存」** 这句话在这里是类型系统保证的，不是风格建议。

发出去之后再想用原值，编译器会拦住：

```rust,compile_fail
use std::sync::mpsc;
use std::thread;

fn main() {
    let (sender, receiver) = mpsc::channel();
    let message = String::from("done");
    thread::spawn(move || sender.send(message).unwrap());
    println!("{message}");
    println!("{:?}", receiver.recv());
}
```

`mpsc` 是 **m**ulti-**p**roducer, **s**ingle-**c**onsumer：发送端可以 clone 出多个，接收端只有一个。

```rust,editable
use std::sync::mpsc;
use std::thread;

fn main() {
    let (sender, receiver) = mpsc::channel::<u16>();

    for status in [200_u16, 404, 503] {
        let sender = sender.clone();          // 每个线程一个发送端
        thread::spawn(move || sender.send(status).unwrap());
    }
    drop(sender);                             // 关键：丢掉最后一个发送端

    // 所有发送端都被丢弃后，recv 返回 Err，迭代自然结束
    let collected: Vec<u16> = receiver.iter().collect();
    println!("收到 {} 条: {collected:?}", collected.len());
}
```

`drop(sender)` 那一行是这段代码的关键：**接收端的迭代在「所有发送端都被丢弃」时结束**。忘了它，`receiver.iter()` 会一直等下去——这是 channel 版最常见的挂起原因。
---

## 4. 共享状态：`Arc<Mutex<T>>` 是两个问题，两个工具

```rust,editable
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let statuses = Arc::new(Mutex::new(Vec::new()));
    let worker_view = Arc::clone(&statuses);

    thread::spawn(move || worker_view.lock().unwrap().push(200))
        .join()
        .unwrap();

    assert_eq!(*statuses.lock().unwrap(), vec![200]);
}
```

`Arc<Mutex<T>>` 看起来像一个东西，其实是两个工具叠在一起，各自回答一个问题：

| 工具 | 回答的问题 | 代价 |
|---|---|---|
| `Arc<T>` | 「这个值**归谁**所有？」——多个线程共同拥有 | 原子引用计数（比 `Rc` 慢一点） |
| `Mutex<T>` | 「**谁可以改**它？」——同一时刻只有一个 | 加锁/解锁，以及死锁的可能 |

缺任何一个都不行：只用 `Arc`，多个线程会同时改同一个 `Vec`；只用 `Mutex`，你没法把它交给第二个线程（所有权只有一个）。

### 锁守卫与「`let _` 陷阱」

`lock()` 返回一个守卫（guard）。**守卫在 `drop` 时解锁**，所以它的生命周期就是临界区：

```rust,ignore
let guard = mutex.lock().unwrap();   // 上锁
guard.push(1);                        // 临界区
// guard 在这里离开作用域，自动解锁

let _ = mutex.lock().unwrap();        // 立刻解锁：这一行结束时锁就没了
```

第二行是所有权章节里 `let _` 与 `let _guard` 的区别在并发里的后果。它不是错误，只是几乎总是**不是你想要的意思**——你本来想「持有锁到作用域结束」，实际写成了「拿到就还回去」。

### 锁中毒

`lock()` 返回 `Result` 而不是守卫，原因很实在：**如果持锁线程 panic 了，它保护的数据可能处于半改状态**。这个状态叫「中毒」（poisoned），后续 `lock()` 会返回 `Err`。

```rust,editable
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let shared = Arc::new(Mutex::new(vec![1_u32]));

    let poisoner = Arc::clone(&shared);
    let _ = thread::spawn(move || {
        let mut guard = poisoner.lock().unwrap();
        guard.push(2);
        panic!("改到一半就炸了");
    })
    .join();

    match shared.lock() {
        Ok(guard) => println!("数据仍然可用: {guard:?}"),
        Err(poisoned) => {
            // 中毒不等于不能用：可以取出数据，也可以主动忽略
            println!("锁已中毒，但数据是 {:?}", poisoned.into_inner());
        }
    }
}
```

生产代码通常用 `.lock().unwrap()` 让中毒直接暴露；只有在「数据即使不一致也能安全重建」时才忽略它。

### 死锁：顺序是设计问题

两把锁、两个线程、相反的获取顺序，就是经典死锁。Rust 的类型系统**不**帮你防这个——它只保证内存安全，不保证不卡住。

处理办法按推荐程度排：

1. **减少锁的数量**：一把锁保护一组相关数据，就不用考虑顺序；
2. **固定顺序**：约定「先 A 后 B」，并在文档里写明；
3. **锁的持有时间尽量短**：临界区里不做 I/O、不发网络请求。

---

## 5. `Send` 与 `Sync`：两个精确的定义

- **`Send`**：值的**所有权**可以安全地移到另一个线程。
- **`Sync`**：`&T` 可以安全地被多个线程共享。

两者都是**自动 trait**：编译器根据字段自动推导，你不能手写实现（写了也是 `unsafe`）。看几个常见类型：

| 类型 | `Send` | `Sync` | 为什么 |
|---|---|---|---|
| `i32`、`String`、`Vec<T>` | ✅ | ✅ | 拥有自己的数据，不含共享可变状态 |
| `&T`（`T: Sync`） | ✅ | ✅ | 共享引用是只读的 |
| `&mut T`（`T: Send`） | ✅ | ❌ | 独占引用已经保证只有一个使用者 |
| `Rc<T>` | ❌ | ❌ | 引用计数不是原子的，两个线程同时 clone 会算错 |
| `RefCell<T>` | ✅ | ❌ | 借用检查在运行期，没有跨线程同步 |
| `Arc<T>` | ✅* | ✅* | 原子计数；是否 Send/Sync 取决于 T |
| `Mutex<T>` | ✅* | ✅* | 锁本身就是同步手段；同样取决于 T |

### 反例：跨线程共享 `Rc<RefCell<Vec<_>>>`

这正是本章开头那个「先预测」的问题：

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

**两个原因各自都足以拒绝它**：`Rc` 的计数不是原子的，`RefCell` 的运行期借用表也不是线程安全的。正确的替代是把两个都换掉：

| 想要 | 单线程写法 | 多线程写法 |
|---|---|---|
| 共享所有权 | `Rc<T>` | `Arc<T>` |
| 内部可变性 | `RefCell<T>` | `Mutex<T>` / `RwLock<T>` |

**不要用 `unsafe` 绕开这个错误。** 编译器拒绝的不是形式，是真实的数据竞争。
---

## 6. 三种形状，按顺序问三个问题

并发里的所有权只有三种形状。**按顺序问，命中就停**：

```text
1. 这份数据能不能只归一个执行者？        → 直接 move（最简单）
2. 能不能把「状态变化」变成消息发出去？   → channel
3. 以上都不行，必须原地共享修改？        → Arc<Mutex<T>> / Arc<RwLock<T>>
```

| 形状 | 适合 | 代价 |
|---|---|---|
| 独占 move | 每个任务处理自己的数据 | 无同步开销 |
| channel | 生产者/消费者、事件流、流水线 | 一次消息传递；容量决定背压 |
| 共享状态 | 多个读者需要看到同一份最新数据 | 锁竞争、死锁风险、临界区越长越慢 |

**大多数人会跳过前两步直接上 `Arc<Mutex<T>>`**，因为它最像其他语言里的做法。但前两种形状没有锁，也就没有死锁和锁竞争——先问一遍是不是必须共享，往往能省掉一整类问题。

项目里两种形状都出现了，各有理由：

- `InMemoryRepository` 用 `tokio::sync::RwLock` 保护内部状态：HTTP handler 需要**读同一份最新数据**，共享是本质需求；
- `HealthChecker` 完全不共享可变状态：它只有 `&self` 和一个内部复用了连接池的 `Client`，并发安全由库自己保证。

---

## 7. 深水区

### 7.1 异步世界里为什么不能用标准库的锁

`std::sync::Mutex` 的守卫不是 `Send`，所以**不能跨越 `.await` 持有**。更糟的是：即使编译通过的位置（不跨 `await`），在异步任务里长时间持锁也会卡住整个执行器线程。

| | `std::sync::Mutex` | `tokio::sync::Mutex` |
|---|---|---|
| 守卫可跨 `.await` | ❌ 编译不过 | ✅ |
| 等待锁时 | 阻塞线程 | 让出执行权，去跑别的任务 |
| 代价 | 更轻 | 更重（要注册 waker） |

选择规则：**临界区里没有 `.await`，用标准库的**（更轻）；**必须跨 `.await` 持锁，用 `tokio::sync` 的**。项目里 `InMemoryRepository` 的读写都在同一个方法内完成，理论上两者都行；用 Tokio 版本是为了让「这不是热点、但绝不能阻塞执行器」这件事在类型上就成立。

### 7.2 什么时候不需要锁

计数器是典型的「看起来需要锁、其实不需要」：

```rust,editable
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

fn main() {
    let completed = Arc::new(AtomicUsize::new(0));

    let mut workers = Vec::new();
    for _ in 0..4 {
        let counter = Arc::clone(&completed);
        workers.push(thread::spawn(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }

    println!("完成数 = {}", completed.load(Ordering::SeqCst));
}
```

原子类型没有锁、没有守卫、没有死锁，只要一次原子指令。**规则：单一数值的更新用原子；多个字段必须一起变化的「不变式」才需要锁。**

### 7.3 更强的工具：作用域线程与线程池

手工 `spawn` + `join` 在需要「固定几个线程反复处理一批任务」时就显得笨重了。`thread::scope` 解决借用问题，线程池（例如 `rayon`）解决调度问题。它们都建立在本章这套所有权规则之上——学到这里，你已经能判断「哪些数据需要 `Arc`、哪些可以直接借用」。

---

## 8. 常见误解

| 误解 | 准确说法 |
|---|---|
| 「`Arc` 就够了，不需要 `Mutex`」 | `Arc` 只解决「归谁所有」；同时修改仍然需要互斥。 |
| 「`Mutex` 会自动让数据变成共享的」 | 锁不转移所有权，你还是得用 `Arc` 把 `Mutex` 交给别的线程。 |
| 「`Rc` 加锁就能跨线程」 | `Rc` 的计数不是原子的，加锁也救不回来；要换 `Arc`。 |
| 「锁中毒了数据就不能用了」 | 中毒只是标记「持锁时 panic 过」，数据仍可取回；是否信任由你决定。 |
| 「`lock().unwrap()` 是坏味道」 | 在「中毒即 bug」的场景里，让它快速失败反而是对的。 |
| 「异步代码里也能随便用同步锁」 | 跨 `.await` 持有会编译失败；不跨也会阻塞执行器线程。 |

---

## 9. 练习与自测

### 练习

完成 `12_threads` 和 `14_send_sync`。第二个练习会直接考你：**跨线程共享 `Rc<RefCell<Vec<_>>>` 会在哪个 trait bound 被拒绝**（答案在第 5 节的报错里）。

```sh
cd exercises
rustlings run 12_threads
rustlings run 14_send_sync
```

然后用项目的测试验证一遍真实并发代码的形状：

```sh
cargo test -p monitor-store --test in_memory
cargo test -p monitor-core --test concurrency
```

### 自测清单（能全部做到才算掌握）

1. 解释 `thread::spawn` 为什么几乎总需要 `move`，并说出对应的错误码。
2. 说出 `join()` 的两个作用，以及不 `join` 会失去什么。
3. 用 `thread::scope` 写一段借用局部变量的并发代码，并说明它为什么不需要 `Arc`。
4. 解释 `mpsc` 名字的含义，以及「所有发送端被丢弃」与接收端迭代结束的关系。
5. 拆解 `Arc<Mutex<T>>`：每个部分各回答什么问题？
6. 给出 `Send` 与 `Sync` 的准确定义，并判断 `&mut T`、`Rc<T>`、`RefCell<T>` 各自是否满足。
7. 解释 `let _ = mutex.lock()` 会发生什么，为什么它多半不是你想要的。
8. 说出「独占 move → channel → 共享状态」这个提问顺序的理由，并举一个跳过前两步反而更糟的例子。

---

## AI 辅导提示词

这三段可以直接复制给 AI 助手（Kimi、ChatGPT 等）。它们的设计意图是**让助手出题和追问，而不是替你写代码**——完整方法论见[用 AI 助手当教练](../guided/ai-tutor.md)。

```text
下面是我的并发代码。请不要直接重写，而是按顺序回答：
1. 这里的数据能不能只归一个线程独占？
2. 能不能改成发消息（channel）而不是共享内存？
3. 如果必须共享，锁的粒度是否可以更小、临界区是否可以更短？
每问给出你的判断和理由。

[粘贴代码]
```

```text
请出 3 道题，考察 Send 与 Sync 的判断：
给定一个结构体定义，让我判断它是否 Send / Sync，以及原因。
先只出题，我答完后逐条点评，重点指出我是靠字段判断还是靠背结论。

[粘贴或让我自己出结构体]
```

```text
我的程序偶发卡死，怀疑是死锁或忘记 drop 发送端。
请先问我三个问题（有几把锁、获取顺序是否一致、channel 的发送端在哪里被丢弃），
再给我一个最小复现实验的设计。不要先看我的代码就要结论。
```
