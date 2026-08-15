# `Send` / `Sync`：异步任务为什么拒绝这个值

| 元数据 | 内容 |
|---|---|
| 任务 | 判断值能否跨任务移动或共享 |
| 概念 | `Send`、`Sync`、`Arc`、锁、`.await` |
| 先修 | ownership、trait、future |
| 项目阶段 | 异步网络 |

Tokio 多线程任务可能在线程之间移动 future。先检查是否跨 `.await` 持有非 `Send` 值或同步锁，再考虑更换类型。先用[线程、消息与共享状态](../async/threads-channels-state.md)建立 trait 直觉，再回到 [Future、任务与运行时](../async/future-runtime.md)。
