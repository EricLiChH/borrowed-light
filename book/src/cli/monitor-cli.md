# CLI 检查点：先建立顺序基线

| 任务 | 概念 | 预计时间 | 项目产物 |
|---|---|---:|---|
| 构建可安装的顺序命令行监测器 | 模块、迭代器、Serde、Clap、退出码、阻塞 I/O | 3 小时 | `monitor-sync` CLI |

> **预计 3 小时**。这一章刻意不用 Tokio、不写 `.await`。理由不是「异步太难」，而是：**没有基线就没有对照物。** 下一章把同一个程序改成并发版本时，你需要一个能回答「变快了吗、行为变了吗」的参照系。

---

## 1. 顺序版本不是简化版，是对照组

```text
monitor-sync target Rust https://www.rust-lang.org   ← 只校验输入，不发请求
monitor-sync check Rust https://www.rust-lang.org    ← 一次阻塞请求
monitor-sync batch targets.json                      ← 逐个请求，慢但可预测
```

这三个命令覆盖了 CLI 的全部职责：**解析参数 → 校验输入 → 做一件事 → 按约定报告结果**。异步版本会替换掉「做一件事」的内部实现，但接口和输出格式必须保持一致——这也是本章反复强调「协议稳定」的原因。

---

## 2. Clap：把参数变成数据

手写参数解析很快就会变成一堆 `std::env::args()` 加 `if`。Clap 的价值在于：**你只声明数据的形状，解析、帮助文本和错误都由它生成。**

真实的帮助输出（不是示意，是运行 `--help` 得到的）：

```text
网站健康监测器的顺序 CLI 检查点

Usage: monitor-sync <COMMAND>

Commands:
  target  Validate a target without making a network request
  check   Check one website with a blocking client
  batch   Check a JSON target list one item at a time
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

对应这份输出的声明，在实际代码里就是这样（`projects/monitor-cli/src/bin/monitor_sync.rs`）：

```rust,ignore
#[derive(Debug, Parser)]
#[command(name = "monitor-sync", about = "网站健康监测器的顺序 CLI 检查点")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate a target without making a network request.
    Target { name: String, url: String },
    /// Check one website with a blocking client.
    Check { name: String, url: String },
    /// Check a JSON target list one item at a time.
    Batch { file: PathBuf },
}
```

三个有意思的地方：

1. **`enum` 直接变成子命令集合**。每个变体是一个子命令，字段是它的参数。这就是「用类型表达结构」的又一例。
2. **文档注释就是帮助文本**。上面 `--help` 里的英文描述，来自 `///` 注释——文档和界面是同一份东西。
3. **`PathBuf` 而非 `String`**。`batch` 的文件参数用路径类型声明意图；Clap 会把它从字符串解析成 `PathBuf`。

参数的三种形态：

| 形态 | 声明 | 出现方式 | 例子 |
|---|---|---|---|
| 位置参数 | `name: String` | `monitor check Rust https://...` | 必填，按顺序 |
| 选项 | `#[arg(long)] timeout_ms: u64` | `--timeout-ms 5000` | 可选，带默认值 |
| 开关 | `#[arg(long)] verbose: bool` | `--verbose` | 存在即为真 |

### 用类型挡住非法输入

```rust,ignore
Check {
    name: String,
    url: String,
    #[arg(long, default_value_t = 5_000)]
    timeout_ms: u64,
    #[arg(long, default_value_t = NonZeroUsize::MIN.saturating_add(1))]
    attempts: NonZeroUsize,
}
```

`attempts` 的类型是 `NonZeroUsize`，所以 `--attempts 0` **在参数解析层就被拒绝**，根本进不了业务代码，退出码是 2：

```text
$ monitor check x http://127.0.0.1:1 --attempts 0
error: invalid value '0' for '--attempts <ATTEMPTS>': number would be zero for non-zero type
```

这是[NonZero 配方](../recipes/nonzero-and-newtypes.md)的同一个思路：**非法值在类型层面不存在，就不用写校验、也不用写测试去覆盖那个分支。**
---

## 3. 退出码：三种结局，不是两种

CLI 和库最大的区别是：**它有一个只有它能决定的东西——进程退出码。**

```text
0  命令跑完了，每个站点都响应了
1  命令跑完了，至少一个站点没响应
2  命令根本没跑起来：参数错、文件读不到、URL 非法
```

三种结局对应三种不同的「谁该被叫醒」：0 什么都不用做，1 是运维问题（站点挂了），2 是使用者问题（命令写错了）。把它们混成「非零」会让脚本无法判断该重试还是该修命令。

真实的失败输出：

```text
$ MONITOR_DISABLE_PROXY=1 monitor check dead http://127.0.0.1:1 --attempts 1
{
  "name": "dead",
  "url": "http://127.0.0.1:1/",
  "reachable": false,
  "status": null,
  "failure": "connect",
  "reason": "error sending request for url (http://127.0.0.1:1/)"
}
EXIT=1
```

而参数错误走的是另一条路：

```text
$ monitor check bad 'not a url'
error: target URL must be an absolute http(s) URL with a host
EXIT=2
```

注意这两次输出去了**不同的流**：

| 流 | 内容 | 谁在看 |
|---|---|---|
| stdout | 检查报告（JSON） | 脚本、管道、下游程序 |
| stderr | 诊断信息（`error: ...`） | 人 |

这个分工不是洁癖。`monitor check ... > report.json` 时，报告进文件、错误留在终端；如果诊断也写到 stdout，你的 JSON 文件就被污染了。

有了这三种结局，脚本可以完全不解析 JSON：

```sh
if monitor check api https://api.example.com --attempts 1 > /dev/null; then
    echo "api 正常"
else
    case $? in
        1) echo "api 挂了，需要处理" ;;
        2) echo "命令写错了" ;;
    esac
fi
```

---

## 4. JSON 是协议，不是调试输出

```text
$ monitor-sync target Rust https://www.rust-lang.org
{
  "name": "Rust",
  "url": "https://www.rust-lang.org/"
}
```

两个细节值得停一下：

1. **URL 多了结尾的斜杠**。这不是 bug，是 URL 标准化的结果：`url` crate 会把空路径补成 `/`。项目的领域类型在构造时做一次规范化，之后所有输出都一致——详见 `docs/adr/0018`。
2. **字段顺序固定、字段名稳定**。`name`、`url`、`reachable`、`status`、`failure`、`reason` —— 这六个字段一旦发布就是对外契约。批量命令的输出是同样的对象组成的一个数组：

```text
$ MONITOR_DISABLE_PROXY=1 monitor batch targets.json --attempts 1
[
  {
    "name": "one",
    "url": "http://127.0.0.1:1/",
    "reachable": false,
    "status": null,
    "failure": "connect",
    "reason": "error sending request for url (http://127.0.0.1:1/)"
  }
]
EXIT=1
```

**为什么 CLI 输出 JSON 而不是彩色表格？** 因为下一个阶段的 Web API 要复用同一套字段。CLI 阶段把协议定下来，异步阶段和 HTTP 阶段就只是在换传输方式——`CheckOutput` 与 Web 的 `CheckView` 至今保持同样的字段名，这不是巧合。

在代码里，协议由 `serde` 的派生宏描述：

```rust,ignore
#[derive(Debug, Serialize)]
pub struct CheckOutput<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub reachable: bool,
    pub status: Option<u16>,
    pub failure: Option<&'static str>,
    pub reason: Option<&'a str>,
}
```

`Serialize` 只写一次，读代码的人一眼就能看出「这个命令对外承诺了什么」。

---

## 5. 阻塞 I/O 的形状

同步版本用一个**复用的**阻塞客户端：

```rust,ignore
/// 阻塞客户端要显式设置超时：顺序检查没有外层 deadline 兜底。
pub fn blocking_client(timeout: Duration) -> Result<reqwest::blocking::Client, reqwest::Error> {
    let mut builder = reqwest::blocking::Client::builder().timeout(timeout);
    if std::env::var_os(DISABLE_PROXY_ENV).is_some() {
        builder = builder.no_proxy();
    }
    builder.build()
}
```

三个要点：

| 要点 | 为什么 |
|---|---|
| 客户端在循环外创建一次 | `Client` 内部持有连接池和 TLS 会话；每次请求新建客户端等于每次重新握手 |
| 必须显式设超时 | 阻塞调用没有外层 `timeout` 包装；不设就是「可能永远不返回」 |
| 顺序批处理就是 `for` | 简单、可预测、容易推理——这正是它作为基线的价值 |

检查函数本身短得几乎不像「网络代码」，这是刻意的：**网络细节被 `reqwest` 吸收，错误分类被 `monitor-core` 的 `classify_transport_error` 吸收**，CLI 只剩下「取一次状态码」这件事。

```rust,ignore
fn check(client: &reqwest::blocking::Client, target: &MonitorTarget) -> CheckResult {
    match client.get(target.url().clone()).send() {
        Ok(response) => CheckResult::reachable(target, response.status().as_u16()),
        Err(error) => CheckResult::unreachable_with(
            target,
            classify_transport_error(&error),
            error.to_string(),
        ),
    }
}
```

注意它返回的是 `CheckResult` 而不是 `Result`：**「站点不可达」不是这个函数的错误，是它的正常输出之一。** 只有「命令没法执行」才用 `Result` 上报。
---

## 6. 一个真实的陷阱：你在测代理，还是在测站点？

写这一章时我在开发机上跑了一次「必定失败」的检查，期待看到 `reachable: false`：

```text
$ monitor check dead http://127.0.0.1:1 --attempts 1
{
  "name": "dead",
  "url": "http://127.0.0.1:1/",
  "reachable": true,
  "status": 502,
  "failure": null,
  "reason": null
}
EXIT=0
```

**一个连不上的地址，报告「可达」，退出码 0。** 原因不是代码写错了，而是这台机器的环境里有代理变量：请求被送到了代理，代理替目标站点「回答」了一个 502。

这件事对健康监测器是致命的：**它测的是代理，不是你关心的站点。**

项目的处理方式是在客户端构建时读一个环境变量：

```rust,ignore
pub const DISABLE_PROXY_ENV: &str = "MONITOR_DISABLE_PROXY";

// ...
if std::env::var_os(DISABLE_PROXY_ENV).is_some() {
    builder = builder.no_proxy();
}
```

加上它之后才是预期行为：

```text
$ MONITOR_DISABLE_PROXY=1 monitor check dead http://127.0.0.1:1 --attempts 1
{ "reachable": false, "failure": "connect", ... }
EXIT=1
```

这个案例值得记住的不是变量名，而是那条更一般的规则：

> **环境变量是隐式依赖。** 只要一段代码读了环境，它的行为就不再只由参数决定——测试必须显式设置环境，否则「在我机器上是好的」会变成常态。项目的集成测试**无条件**设置 `MONITOR_DISABLE_PROXY`，而不是「需要时才设」。

---

## 7. 让 CLI 可测试

命令行的逻辑在 `run` 函数里，看起来可以单元测试。但真正要验证的东西——参数解析、退出码、stdout 与 stderr 的分流——只有**真的把它当进程跑起来**才测得到。

Cargo 为此提供了一个环境变量：`CARGO_BIN_EXE_<二进制名>`，它给出该 crate 的二进制路径。

```rust,ignore
use std::process::Command;

fn monitor(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_monitor"))
        .env("MONITOR_DISABLE_PROXY", "1")   // 隐式输入必须显式控制
        .args(arguments)
        .output()
        .expect("monitor binary should run")
}

#[test]
fn a_failed_check_exits_with_code_one_and_still_prints_json() {
    // 本机 1 端口会立即拒绝连接：不触网、不需要超时
    let output = monitor(&["check", "dead", "http://127.0.0.1:1", "--attempts", "1"]);

    assert_eq!(output.status.code(), Some(1), "检查失败不是崩溃");
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout 仍然必须是 JSON");
    assert_eq!(json["reachable"], false);
}

#[test]
fn an_invalid_target_exits_with_code_two() {
    let output = monitor(&["check", "bad", "not a url"]);

    assert_eq!(output.status.code(), Some(2), "用法错误是退出码 2");
    assert!(output.stdout.is_empty(), "用法错误不污染 stdout");
}
```

三个断言各测一层：`status.code()` 测退出码协议，`stdout` 测报告协议，`stdout.is_empty()` 测流向分工。

**代价与取舍：** 每个这样的测试都要启动一个进程（毫秒级）。所以 CLI 测试要**少而准**——每个契约一条，而不是把逻辑测试都搬到这里。逻辑测试仍然留在库函数里，快得多。

---

## 8. 深水区

### 8.1 派生宏还是 builder

Clap 有两种用法：`#[derive(Parser)]` 和链式 `Command::new(...).arg(...)`。项目的选择是派生宏，理由是**参数结构就是数据结构**：`enum Command` 一眼能看出有几个子命令、各自要什么。当参数需要动态生成（比如从配置文件的键生成选项）时，builder 才更合适。

### 8.2 可安装性

一个 CLI 只有能被装到 `PATH` 里才算完成：

```sh
cargo install --path projects/monitor-cli --bin monitor-sync
cd /tmp && monitor-sync --help     # 从别的目录调用，验证没有依赖当前目录
cargo uninstall monitor-sync
```

这一步能暴露两类问题：硬编码的相对路径（`targets.json` 这类参数必须是用户传入的），以及把「开发目录下的资源」当成理所当然。

### 8.3 错误信息写给谁

| 读者 | 写在哪里 | 该长什么样 |
|---|---|---|
| 人 | stderr，前缀 `error:` | 说清**哪一步**失败、**怎么改** |
| 脚本 | 退出码 | 三种结局，不解析文本 |
| 下游程序 | stdout 的 JSON | 字段稳定、可选字段用 `null` 而不是省略 |

项目的 `TargetError` 的 `Display` 实现就是按第一行写的：`"target URL must be an absolute http(s) URL with a host"` —— 它描述的是**要求**，而不是「解析失败」。前者用户能照着改，后者只能让人困惑。
---

## 9. 设计检查：四个职责，各归其位

回头看你写的这一章，每一层只做一件事：

| 层 | 职责 | 不该做的事 |
|---|---|---|
| Clap | 把参数变成数据 | 校验业务规则 |
| `MonitorTarget` | 守住输入不变量（非空名字、http(s)、可解析） | 发网络请求 |
| `check()` | 一次阻塞 HTTP 往返，返回 `CheckResult` | 决定退出码、打印输出 |
| `main` | 把结果翻译成 stdout 与退出码 | 处理网络错误 |

**判断分层是否正确的办法：如果明天把 CLI 换成 Web 服务，哪一层需要改？** 按上表，只有 `main` 那一层——另外三层原封不动。项目正是这样做的：`monitor-core` 和 `monitor-domain` 被 CLI 和 Web 共用。

---

## 10. 常见误解

| 误解 | 准确说法 |
|---|---|
| 「退出码非零就是出错」 | 1 是「命令跑完了，站点挂了」，2 才是「命令没跑起来」。脚本靠这个区分重试和修命令。 |
| 「错误都打到 stdout 方便看」 | 诊断走 stderr，报告走 stdout。否则 `> report.json` 会把 JSON 弄脏。 |
| 「每次请求新建 client 更干净」 | 会丢掉连接池和 TLS 会话；一个 Client 复用到底。 |
| 「阻塞调用不需要设超时」 | 没有外层 deadline 兜底，不设就是可能永不返回。 |
| 「输出是给人看的，改格式无所谓」 | 六个字段是对外协议；异步迁移和 Web API 都依赖它稳定。 |
| 「测试环境有代理不影响本地请求」 | 会。本机地址一样会被代理接走，报告出一个假的可达状态。 |

---

## 11. 练习与实验

先跑通现有的集成测试，看清「进程级测试」长什么样：

```sh
cargo test -p monitor-cli --test sync_command
cargo test -p monitor-cli --test exit_codes
```

完成 `06_summary` 后，用迭代器统计一批结果中 2xx 的数量。

以下五个实验计入标准路径。**每个实验都先写失败测试或明确预测，再改实现**：

1. **输入边界（60–90 分钟）**：分别运行空名称、非 HTTP URL、缺失参数三种失败，记录错误来自 Clap 还是 `MonitorTarget`，以及各自的退出码。
2. **错误传播（90 分钟）**：给不存在的 JSON 文件和内容非法的 JSON 各写一个集成测试，确认错误写到 stderr、退出码非零、stdout 为空。
3. **顺序证据（90 分钟）**：让第一个本地服务器等待、第二个立即响应，确认 `monitor-sync batch` 仍会被第一个阻塞——这就是下一章要改进的那条基线。
4. **协议稳定（120 分钟）**：为 `target`、`check`、`batch` 的 JSON 字段写断言。这些断言将在异步迁移后原样复用。
5. **可安装性（60 分钟）**：`cargo install --path projects/monitor-cli --bin monitor-sync`，从另一个目录调用帮助，再卸载。

下一章会把顺序实现迁移到 Tokio；此时无需提前阅读异步二进制。

---

## 12. 自测清单（能全部做到才算掌握）

1. 说出三种退出码各自的含义，并写一段靠退出码分支、不解析 JSON 的 shell。
2. 解释 stdout 与 stderr 的分工，以及混用会造成什么后果。
3. 说明为什么 `--attempts 0` 会在参数解析层就被拒绝，这替代了哪一段校验代码。
4. 说出复用 `Client` 的两个好处，以及不设超时会发生什么。
5. 用 `CARGO_BIN_EXE_<name>` 写一个集成测试，断言退出码与 stdout。
6. 解释「本机地址的检查返回 502」是怎么发生的，以及项目如何避免它。
7. 说出 `CheckResult` 与 `Result` 在 `check()` 里的分工：为什么前者不是错误。
8. 给定一个新需求（例如「只输出不可达的站点」），说明四个职责层里哪几层需要改。

---

## AI 辅导提示词

这三段可以直接复制给 AI 助手（Kimi、ChatGPT 等）。它们的设计意图是**让助手出题和追问，而不是替你写代码**——完整方法论见[用 AI 助手当教练](../guided/ai-tutor.md)。

```text
下面是我的 CLI 的 --help 输出和参数结构。请帮我审查：
1. 哪些参数其实应该用更强的类型（而不是 String/u64）来表达？
2. 哪些非法输入现在只能在运行期被发现？
请先给分析，不要直接改代码。

[粘贴 --help 输出与 Cli/Command 定义]
```

```text
我写了一段 shell 脚本调用这个 CLI。请检查我是否在解析 JSON 文本、
或者依赖了 stderr 的内容——如果是，请告诉我该改成依赖退出码的哪一部分。
不要重写脚本，只指出问题所在的那几行。

[粘贴脚本与 CLI 的输出样本]
```

```text
请为这个命令行工具出 3 道题，考察「进程边界」的行为：
退出码、stdout/stderr 分流、以及环境变量带来的隐式依赖。
先只出题，我答完后逐条点评，并指出我理由里的错误。

[粘贴命令列表]
```
