# 欢迎回来

这不是从 `Hello, world!` 开始重读一遍语法。你曾经碰过 Rust；这条路径先诊断哪些知识仍然在，再把模糊的部分恢复成能用于项目的能力。

完整标准路径预计 **30–40 小时**（换算成周数见[标准路径](guided/standard-path.md)），首个所有权样章需要 60–90 分钟。

---

## 你会构建什么

一条贯穿全程的主线：**网站健康监测器**。它从命令行工具长成一个带持久化的并发服务，每一阶段都保留上一阶段的接口契约。

```text
阶段一  顺序 CLI
  $ monitor-sync check Rust https://www.rust-lang.org
  {"name":"Rust","url":"https://www.rust-lang.org/","reachable":true,"status":200}

阶段二  并发检查
  $ monitor batch targets.json --concurrency 8     ← 同一个 JSON 协议，加入并发窗口与总期限

阶段三  持久化 HTTP 服务
  $ curl -X POST localhost:3000/targets -d '{"name":"self","url":"http://localhost:3000/healthz"}'
  HTTP/1.1 201 Created
  $ sqlite3 monitor.db 'SELECT * FROM check_results;'
  1|1|204||
```

三个阶段共用同一套领域类型、错误分类和退出码约定。**换的是传输方式，不是语义**——这正是本书反复训练的判断力。

---

## 这本书怎么组织

| 部分 | 章节 | 你要获得的产物 |
|---|---|---|
| 导论 | [入门诊断](guided/diagnostic.md)、[AI 助教](guided/ai-tutor.md)、[标准路径](guided/standard-path.md) | 一条属于你的起点与节奏 |
| 核心能力 | [所有权](guided/ownership.md)、[值/枚举/集合](core/values-collections-modules.md)、[Option/Result](core/result-option.md)、[生命周期/泛型/闭包](core/lifetimes-generics-closures.md)、[trait 与测试](core/traits-and-tests.md) | 领域模型 + 可替换接缝 + 契约测试 |
| CLI | [顺序检查器](cli/monitor-cli.md) | 可安装、可脚本化、协议稳定的命令行工具 |
| 异步 | [线程](async/threads-channels-state.md)、[Future 与运行时](async/future-runtime.md)、[channel 与取消](async/channels-cancellation.md)、[reqwest](async/http-checker.md)、[受控并发与重试](async/concurrency-retry.md) | 有并发窗口、总期限、有限重试的检查器 |
| Web | [Axum](web/axum-api.md)、[SQLite](web/sqlite-storage.md)、[毕业检查点](web/final-checkpoint.md) | 持久化的 HTTP 服务，含状态码与优雅关闭 |
| 速查 | [语法速查](reference/syntax.md)、[配方索引](recipes/index.md) | 卡住时随手翻的两页 |

---

## 每章的统一结构

深章节都按同一套骨架写，你知道该往哪儿看：

| 段落 | 作用 |
|---|---|
| 引言 | 这一章解决什么问题，预计多久，可以跳过什么 |
| 正文 | 从问题出发的心智模型 + 可运行代码 |
| 真实报错 | 编译器原话，不是转述 |
| 深水区 | 边界情况、代价、与其它语言的对比 |
| 常见误解 | 一张表，破除似是而非的说法 |
| 练习 | rustlings 或项目任务，附验收标准 |
| 自测清单 | 不看资料能否回答 |
| AI 辅导提示词 | 三段可直接复制的提问模板 |

**代码是被验证的**：书中每个可运行例子都由 `mdbook test` 编译，每个"不能编译"的反例都被验证过确实失败。

---

## 三种进入方式

| 你的状态 | 从哪开始 |
|---|---|
| 想系统恢复 | [入门诊断](guided/diagnostic.md) → [标准路径](guided/standard-path.md) |
| 正在被某个报错卡住 | [配方索引](recipes/index.md)，按错误码进入 |
| 想边做边学 | [所有权](guided/ownership.md) 的三个实验 → 直接进 [CLI](cli/monitor-cli.md) |

---

## 本地练习

```sh
cd exercises
rustlings
```

Rustlings 会自动重跑当前练习。输入 `h` 获得方向提示；完成后会显示参考解的位置。

项目本身的验收靠测试，不靠肉眼：

```sh
cargo test --workspace      # 56 个测试
scripts/check.sh            # 格式、clippy、测试、链接、书、练习
```

## 配一个 AI 助教

本书假定你手边有一个对话式助手。它能让学习快很多，也能让你更慢——区别在于你让它演什么角色。[用 AI 助手当教练](guided/ai-tutor.md)给了六种提问模式和每章三段现成提示词，核心只有一条：**不要在你想清楚之前让它给完整代码**。

> 源码已经公开，教材网站仍未部署。学习进度只保存在你的电脑上，不需要账户。
