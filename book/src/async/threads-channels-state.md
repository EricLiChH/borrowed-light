# 线程、消息与共享状态

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 比较三种并发所有权形状 | thread、move closure、mpsc、Arc、Mutex、Send、Sync | 150 分钟 | 并发地基 |

先用标准库线程看见所有权问题，再进入 async。`thread::spawn` 的闭包可能比当前函数活得更久，因此通常需要 `move` 接管捕获值。

```rust
use std::thread;

fn main() {
    let target = String::from("https://example.com");
    let worker = thread::spawn(move || target.len());
    assert_eq!(worker.join().unwrap(), 19);
}
```

## 消息传递：移动值，不共享容器

```rust
use std::{sync::mpsc, thread};

fn main() {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || sender.send(204_u16).unwrap());
    assert_eq!(receiver.recv().unwrap(), 204);
}
```

发送后值的所有权进入 channel。接收者不需要锁住发送者的 Vec，也不会同时修改同一位置。

## 共享状态：共享所有权 + 互斥访问

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let statuses = Arc::new(Mutex::new(Vec::new()));
    let worker_view = Arc::clone(&statuses);
    thread::spawn(move || worker_view.lock().unwrap().push(200)).join().unwrap();
    assert_eq!(*statuses.lock().unwrap(), vec![200]);
}
```

- `Send`：值的所有权可以安全移到另一个线程。
- `Sync`：`&T` 可以安全被多个线程共享。
- `Arc<T>` 只解决共享所有权；内部可变性仍需 `Mutex`/`RwLock` 或消息传递。

先预测：跨线程共享 `Rc<RefCell<Vec<_>>>` 会在哪个 trait bound 被拒绝？不要用 `unsafe` 绕开；改成符合并发语义的类型。完成 `12_threads` 和 `14_send_sync`。
