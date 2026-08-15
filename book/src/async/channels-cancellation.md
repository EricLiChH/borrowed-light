# Channel、取消与背压

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 让异步任务能暂停、排队和停止 | Tokio task、mpsc、backpressure、cancellation safety | 120 分钟 | 有边界的任务流水线 |

Tokio 的有界 channel 把容量写进系统：队列满时 `send().await` 会暂停生产者，这就是背压（backpressure），不是错误。

```rust,ignore
use tokio::sync::mpsc;

let (sender, mut receiver) = mpsc::channel::<String>(8);
tokio::spawn(async move {
    sender.send("https://example.com".into()).await.unwrap();
});
while let Some(url) = receiver.recv().await {
    println!("checking {url}");
}
```

## 取消发生在 `.await` 边界

丢弃一个 future 会取消尚未完成的工作。设计时逐个检查 `.await`：在它之前是否已经修改了半份状态？是否跨暂停点持有同步锁？是否必须用 guard/事务保证恢复？

```rust,ignore
tokio::select! {
    result = checker.check(&target) => save(result).await?,
    _ = shutdown.recv() => return Ok(()),
}
```

`select!` 结束后，未选中的分支会被丢弃。网络请求通常可取消；“扣款后再写记录”这类两步副作用则必须重新设计。

## 项目里的三种边界

1. `buffer_unordered(N)` 限制同时进行的目标数。
2. 每个目标的总超时限制请求、重试和退避的总预算。
3. Web 服务的 Ctrl-C 信号停止接收新连接，并等待在途任务结束。

完成 `13_future`：亲手轮询一个立即就绪的 future；再修改其返回逻辑，让失败从运行期 panic 变成测试反馈。
