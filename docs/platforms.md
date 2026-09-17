# macOS、Linux 与 Windows 本地验证

仓库锁定 Rust 1.94.0；完整维护检查还需要 Python 3。先安装 Rustup，再安装课程工具：

```sh
cargo install mdbook --version 0.5.4 --locked
cargo install rustlings --version 6.5.0 --locked
```

## macOS / Linux

```sh
sh scripts/check.sh
mdbook serve book --open
```

环境变量放在命令前：

```sh
DATABASE_URL=sqlite://monitor.db BIND_ADDRESS=127.0.0.1:3000 cargo run -p monitor-web
```

## Windows PowerShell

```powershell
./scripts/check.ps1
mdbook serve book --open
```

PowerShell 通过 `$env:` 设置当前终端的环境变量：

```powershell
$env:DATABASE_URL = "sqlite://monitor.db"
$env:BIND_ADDRESS = "127.0.0.1:3000"
cargo run -p monitor-web
```

`scripts/check.sh` 需要 POSIX shell；Windows 原生流程使用 `scripts/check.ps1`。两个入口都依次执行格式、Clippy、工作区测试、内部链接、mdBook 测试/构建和 Rustlings 检查。GitHub Actions 在 Ubuntu、macOS 和 Windows 上分别运行对应入口，但不部署 Pages。

## 常见差异

| 任务 | macOS / Linux | Windows PowerShell |
|---|---|---|
| 设置环境变量 | `NAME=value command` | `$env:NAME = "value"` 后运行命令 |
| 行续接 | `\` | 反引号 `` ` ``，或写成单行 |
| 完整检查 | `sh scripts/check.sh` | `./scripts/check.ps1` |
| 调用本地服务 | `curl` | 推荐 `curl.exe`，避免旧版 PowerShell 别名差异 |

自动测试只使用回环地址、虚拟时间和临时内存数据库，因此不要求公网、Docker 或本机 SQLite 命令行工具。
