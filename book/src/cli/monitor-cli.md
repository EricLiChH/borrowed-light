# CLI 检查点：从一个目标到批量检查

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 构建可安装的命令行监测器 | 模块、迭代器、Serde、Clap、退出码 | 150 分钟 | `monitor` CLI |

先确认目标输入，不访问网络：

```sh
cargo run -p monitor-cli --bin monitor -- target Rust https://www.rust-lang.org
```

检查一个站点：

```sh
cargo run -p monitor-cli --bin monitor -- check Rust https://www.rust-lang.org --attempts 2
```

批量配置 `targets.json`：

```json
[
  {"name": "Rust", "url": "https://www.rust-lang.org"},
  {"name": "Example", "url": "https://example.com"}
]
```

```sh
cargo run -p monitor-cli --bin monitor -- batch targets.json --concurrency 4
```

真实公网只用于手动演示。自动测试启动本机临时 HTTP 服务，因此断网时也能稳定运行：

```sh
cargo test -p monitor-cli
```

## 设计检查

- Clap 只负责把参数变成数据。
- `MonitorTarget` 守住输入不变量。
- `HealthChecker` 隐藏网络调度。
- 输出使用 JSON，便于管道、脚本和后续 Web 复用概念。

完成 `06_summary` 后，尝试用迭代器统计一批结果中 2xx 的数量。

## CLI 阶段实验（计入标准路径）

按顺序做完以下实验，把本章从“看懂命令”扩展为 8–10 小时的可交付阶段：

1. **输入边界（60–90 分钟）**：运行 `monitor target` 的空名称、非 HTTP URL、缺失参数三种失败；记录校验来自 Clap 还是 `MonitorTarget`。
2. **错误传播（90 分钟）**：给不存在的 JSON 文件和非法 JSON 各写一个集成测试，确认错误写到 stderr 且退出码非零。
3. **顺序版本（90 分钟）**：先用普通 `for` 循环逐个 `.await`，记录结果顺序；它是“异步函数但没有并发”的基线。
4. **批量版本（120 分钟）**：恢复 `check_all`，用本地服务器证明并发受限且输出仍按输入顺序。
5. **可安装性（60 分钟）**：运行 `cargo install --path projects/monitor-cli`，从另一个目录调用 `monitor --help`，然后卸载本地二进制。

这些实验都应先写失败测试或明确预测，再改实现；不要用真实公网是否恰好可达作为完成证据。
