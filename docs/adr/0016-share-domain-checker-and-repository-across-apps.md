# 让 CLI 与 Web 共享领域和检查边界

网站健康监测器按职责拆成四个深模块：`monitor-domain` 维护不变量与结果分类，`monitor-core` 隐藏 reqwest、总超时、重试和并发调度，`monitor-store` 隐藏内存/SQLite 的持久化差异，CLI 与 Web 只负责各自的输入输出协议。CLI 和 Web 共享前两个模块；存储模块由 Web 与对应测试使用。

这样拆分不是按技术名词增加层数，而是让每个公开接口吸收一组容易变化的复杂度。CLI 和 Axum handler 不复制网络分类；路由测试能替换存储；SQLite migration 与行映射不会泄漏到 HTTP 层。代价是小项目拥有多个 crate 和一个异步 trait，但这些边界正是课程需要练习的所有权、错误和 `Send + Sync` 接缝。

当某项功能只被一个应用使用、也没有隐藏显著复杂度时，不为“架构整齐”继续拆 crate。
