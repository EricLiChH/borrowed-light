# CLI 检查点：先建立顺序基线

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 构建可安装的顺序命令行监测器 | 模块、迭代器、Serde、Clap、退出码、阻塞 I/O | 150 分钟 | `monitor-sync` CLI |

这个阶段刻意不要求 Tokio 或 `.await`。`monitor-sync` 使用一个可复用的阻塞 reqwest Client，批量命令用普通 `for`/迭代器逐个完成；它为下一阶段提供可测量的行为基线。

先确认目标输入，不访问网络：

```sh
cargo run -p monitor-cli --bin monitor-sync -- target Rust https://www.rust-lang.org
```

检查一个站点：

```sh
cargo run -p monitor-cli --bin monitor-sync -- check Rust https://www.rust-lang.org
```

批量配置 `targets.json`：

```json
[
  {"name": "Rust", "url": "https://www.rust-lang.org"},
  {"name": "Example", "url": "https://example.com"}
]
```

```sh
cargo run -p monitor-cli --bin monitor-sync -- batch targets.json
```

真实公网只用于手动演示。自动测试会启动本机临时 HTTP 服务，并通过 `MONITOR_DISABLE_PROXY` 禁止继承代理，因此断网或配置了公司代理时仍能稳定运行：

```sh
cargo test -p monitor-cli --test sync_command
```

## 设计检查

- Clap 只负责把参数变成数据。
- `MonitorTarget` 守住输入不变量。
- 顺序检查函数只负责一次阻塞 HTTP 往返。
- 输出使用 JSON，便于管道、脚本和后续异步版本复用协议。

完成 `06_summary` 后，用迭代器统计一批结果中 2xx 的数量。

## CLI 阶段实验（计入标准路径）

1. **输入边界（60–90 分钟）**：运行空名称、非 HTTP URL、缺失参数三种失败；记录校验来自 Clap 还是 `MonitorTarget`。
2. **错误传播（90 分钟）**：给不存在的 JSON 文件和非法 JSON 各写一个集成测试，确认错误写到 stderr 且退出码非零。
3. **顺序证据（90 分钟）**：让第一个本地服务器等待、第二个立即响应，确认 `monitor-sync batch` 仍会被第一个阻塞。
4. **协议稳定（120 分钟）**：为 `target`、`check`、`batch` 的 JSON 字段写断言；后续异步迁移不得随意改协议。
5. **可安装性（60 分钟）**：运行 `cargo install --path projects/monitor-cli --bin monitor-sync`，从另一个目录调用帮助，再卸载本地二进制。

这些实验都先写失败测试或明确预测，再改实现。下一章会把顺序实现迁移到 Tokio；此时无需提前阅读异步二进制。
