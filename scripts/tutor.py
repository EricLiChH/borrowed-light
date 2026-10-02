#!/usr/bin/env python3
"""Real-time Rust tutor backed by the Kimi API.

The book ships three copy-paste prompts per chapter (see
book/src/guided/ai-tutor.md). This script turns them into a command: it reads
the chapter you are on, sends it as context, and streams the answer back.

Usage:
    python3 scripts/tutor.py --check
    python3 scripts/tutor.py ask --chapter ownership "move 之后为什么原绑定不能用了？"
    python3 scripts/tutor.py chat --chapter result-option
    python3 scripts/tutor.py review --code my_snippet.rs

The key is read from .env (KIMI_API_KEY), never from this file.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BOOK = ROOT / "book" / "src"
WORK = ROOT / "work"

DEFAULT_BASE_URL = "https://api.kimi.com/coding/v1"
DEFAULT_MODEL = "kimi-for-coding"
DEFAULT_MODE = "coach"

MAX_CHAPTER_CHARS = 120_000

MODES = {
    "coach": (
        "按教练模式回答。优先给问题和方向，不要直接给完整实现。"
        "如果学习者明显卡住，给出最小提示，而不是整段代码。"
    ),
    "explain": (
        "按讲解模式回答，顺序固定为：1) 没有这个概念时什么问题无法表达；"
        "2) 它引入的代价；3) 一个用错的例子与正确写法；4) 标准库或本项目里的真实用例。"
    ),
    "review": (
        "按代码评审模式回答。以 clippy::pedantic 的口味评审，按严重程度排序，最多五条。"
        "每条给出：问题、为什么在这个项目里是问题、最小修改方向。不要重写整个函数。"
    ),
    "quiz": (
        "按出题模式回答。出五道判断或选择风格的题，考察边界情况而不是定义，"
        "例如这段代码能编译吗、这个函数的复杂度来自哪里。先只出题，等我回答后再点评。"
    ),
    "debug": (
        "按诊断模式回答，只做三件事：1) 用一句话说明编译器认为发生了什么；"
        "2) 指出我的哪个假设是错的；3) 给一个验证该假设的小实验。不要给完整修复。"
    ),
}

CONTEXT = """你在辅导一位学习者阅读 borrowed-light 这本中文 Rust 教材。

学习者背景：以前学过 Rust，但遗忘了大半，现在要重新上手并深入。
工具链：Rust 1.99，edition 2024。
教材的立场：练习是章节的出口，不是章节的替代品；每个结论都要能被编译或测试验证；
     说到 API 时给出可以在 cargo doc 或 docs.rs 里查到的路径。

回答要求：
- 用中文；代码、API 名与编译器报错保持英文原文。
- 涉及编译器行为时，说清是哪一类错误（例如 E0382），并解释编译器在保护什么。
- 不确定的地方直接说不确定，不要编造 API。
"""


def load_env() -> dict[str, str]:
    """Read .env into the environment without overwriting real variables."""
    env_path = ROOT / ".env"
    values: dict[str, str] = {}
    if env_path.is_file():
        for line in env_path.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            key, _, value = line.partition("=")
            values[key.strip()] = value.strip()
    for key, value in values.items():
        os.environ.setdefault(key, value)
    return values


def find_chapter(name: str) -> Path | None:
    """Resolve a chapter by file name, stem, or path fragment."""
    if not name:
        return None
    candidate = Path(name)
    if candidate.is_file():
        return candidate
    matches = sorted(BOOK.rglob("*.md"))
    stem = candidate.stem
    for path in matches:
        if path.stem == stem:
            return path
    for path in matches:
        if name in str(path.relative_to(BOOK)):
            return path
    return None


def build_messages(args) -> list[dict[str, str]]:
    system = CONTEXT + "\n" + MODES[args.mode]
    chapter = find_chapter(args.chapter) if args.chapter else None
    if chapter:
        text = chapter.read_text(encoding="utf-8")[:MAX_CHAPTER_CHARS]
        system += (
            "\n\n以下是学习者正在读的章节，回答时请与它保持一致；"
            "如果学习者的问题与该章节无关，也照常回答：\n\n"
            "===== " + str(chapter.relative_to(ROOT)) + " =====\n" + text
        )

    messages = [{"role": "system", "content": system}]
    if args.code:
        code = Path(args.code).read_text(encoding="utf-8")
        messages.append({"role": "user", "content": "这是我写的代码：\n\n" + code})
    messages.append({"role": "user", "content": args.question})
    return messages


def stream_reply(base_url: str, api_key: str, model: str, messages, show_reasoning: bool):
    """Yield text chunks from the API, printing them as they arrive."""
    # 注意：kimi-for-coding 只接受默认 temperature（传 0.3 会被 400 拒绝）。
    body = json.dumps({"model": model, "messages": messages, "stream": True}).encode(
        "utf-8"
    )
    request = urllib.request.Request(
        base_url.rstrip("/") + "/chat/completions",
        data=body,
        headers={
            "Authorization": "Bearer " + api_key,
            "Content-Type": "application/json",
            "Accept": "text/event-stream",
        },
        method="POST",
    )

    answer: list[str] = []
    reasoning: list[str] = []
    try:
        response = urllib.request.urlopen(request, timeout=300)
    except urllib.error.HTTPError as error:
        detail = error.read().decode("utf-8", "replace")
        print("\n请求被拒绝：HTTP " + str(error.code), file=sys.stderr)
        print(detail[:500], file=sys.stderr)
        return "", ""
    with response:
        for raw in response:
            line = raw.decode("utf-8", "replace").strip()
            if not line.startswith("data:"):
                continue
            payload = line[5:].strip()
            if payload == "[DONE]":
                break
            try:
                chunk = json.loads(payload)
            except json.JSONDecodeError:
                continue
            choices = chunk.get("choices") or []
            if not choices:
                continue
            delta = choices[0].get("delta") or {}
            if show_reasoning and delta.get("reasoning_content"):
                reasoning.append(delta["reasoning_content"])
                sys.stdout.write(delta["reasoning_content"])
                sys.stdout.flush()
            if delta.get("content"):
                answer.append(delta["content"])
                sys.stdout.write(delta["content"])
                sys.stdout.flush()
    print()
    return "".join(answer), "".join(reasoning)


def save_transcript(messages, answer: str) -> Path:
    WORK.mkdir(exist_ok=True)
    from datetime import datetime

    stamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    path = WORK / ("tutor-" + stamp + ".md")
    lines = ["# 辅导记录 " + stamp, ""]
    for message in messages:
        lines.append("## " + message["role"])
        lines.append("")
        lines.append(message["content"])
        lines.append("")
    lines.append("## assistant")
    lines.append("")
    lines.append(answer)
    path.write_text("\n".join(lines), encoding="utf-8")
    return path


def check(base_url: str, api_key: str) -> int:
    request = urllib.request.Request(
        base_url.rstrip("/") + "/models",
        headers={"Authorization": "Bearer " + api_key},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            payload = json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as error:
        print("密钥被拒绝：HTTP " + str(error.code), file=sys.stderr)
        print(error.read().decode("utf-8", "replace")[:300], file=sys.stderr)
        print(
            "\n检查三件事：1) .env 里的 KIMI_API_KEY 是否最新；"
            "2) KIMI_BASE_URL 是否与该密钥的产品面一致（Kimi Code 密钥用 "
            "https://api.kimi.com/coding/v1）；3) 密钥是否已在控制台轮换。",
            file=sys.stderr,
        )
        return 1
    except urllib.error.URLError as error:
        print("无法连接 " + base_url + "：" + str(error.reason), file=sys.stderr)
        return 1

    names = [model.get("id", "?") for model in payload.get("data", [])]
    print("端点 OK：" + base_url)
    print("可用模型：" + ", ".join(names))
    return 0


def main() -> int:
    load_env()

    parser = argparse.ArgumentParser(description="Rust 教材实时辅导（Kimi 驱动）")
    parser.add_argument("command", nargs="?", default="ask", choices=["ask", "chat", "check"])
    parser.add_argument("--check", action="store_true", help="检查密钥与端点后退出（等同 check）")
    parser.add_argument("question", nargs="?", default="", help="要问的问题")
    parser.add_argument("--chapter", default="", help="章节名或路径，作为上下文一起发送")
    parser.add_argument("--mode", default=DEFAULT_MODE, choices=sorted(MODES))
    parser.add_argument("--code", default="", help="要一起发送的代码文件")
    parser.add_argument("--show-reasoning", action="store_true", help="同时显示模型的推理过程")
    parser.add_argument("--no-save", action="store_true", help="不写入 work/ 记录")
    parser.add_argument("--model", default=os.environ.get("KIMI_MODEL", DEFAULT_MODEL))
    parser.add_argument("--base-url", default=os.environ.get("KIMI_BASE_URL", DEFAULT_BASE_URL))
    args = parser.parse_args()

    api_key = os.environ.get("KIMI_API_KEY", "").strip()
    if not api_key:
        print(
            "缺少 KIMI_API_KEY。\n"
            "  1) cp .env.example .env\n"
            "  2) 在 .env 里填入 Kimi 开放平台的密钥\n"
            "  3) python3 scripts/tutor.py --check",
            file=sys.stderr,
        )
        return 1

    if args.command == "check" or args.check:
        return check(args.base_url, api_key)

    if args.command == "chat":
        print("交互模式：输入问题，空行结束。Ctrl-C 退出。", file=sys.stderr)
        while True:
            try:
                question = input("\n你: ").strip()
            except (EOFError, KeyboardInterrupt):
                print()
                return 0
            if not question:
                return 0
            args.question = question
            messages = build_messages(args)
            print("助教: ", end="")
            stream_reply(args.base_url, api_key, args.model, messages, args.show_reasoning)

    if not args.question:
        parser.error("缺少问题；例如：python3 scripts/tutor.py ask --chapter ownership \"为什么……\"")

    messages = build_messages(args)
    answer, _ = stream_reply(
        args.base_url, api_key, args.model, messages, args.show_reasoning
    )
    if not args.no_save and answer:
        path = save_transcript(messages, answer)
        print("\n已记录到 " + str(path.relative_to(ROOT)), file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
