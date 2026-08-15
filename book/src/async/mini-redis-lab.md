# mini-redis 迁移实验

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 阅读并改造一个真实异步协议片段 | frame、task、channel、shutdown | 90–120 分钟 | 设计对照笔记 |

本实验不复制上游代码，也不把 Redis 变成第二个主项目。联网时打开 [Tokio mini-redis 教程](https://tokio.rs/tokio/tutorial/setup)，只完成 setup、spawn 与 channels 三节；离线时先用本仓库前两章的 channel 示例完成预测。

## 三次定向修改

1. 把一个连接一个 task 的边界画出来：哪些值 move 进 task，哪些通过 `Arc` 共享？
2. 将无界 channel 改成容量 8 的有界 channel，记录发送者在队列满时如何表现。
3. 增加 shutdown 信号；在每个 `.await` 标出取消后是否会留下半完成状态。

## 迁移回健康监测器

把观察映射回本项目：连接 task 对应单目标检查；管理者 channel 对应待检查目标；有界容量对应并发/队列预算；shutdown 对应 Axum 优雅关闭。写一页对照，不实现 Redis 数据库。

完成证据不是“服务器跑起来”，而是能回答：为什么一个共享 `HashMap` 适合由单独 manager task 独占，而 HTTP 检查适合由多个受限 task 独立推进？
