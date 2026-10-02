"""Fail when a tracked file looks like it contains a credential.

The gitignore rules keep new files out of the repository; this check covers the
other half — a secret that was pasted into a file which is already tracked, or
written into one before the ignore rule existed. It runs in scripts/check.sh
and therefore in CI.

Only tracked files are scanned, so the local .env is not reported: it is
supposed to hold a real key, and it is ignored on purpose.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PATTERNS: list[tuple[re.Pattern[str], str]] = [
    (re.compile(r"sk-kimi-[A-Za-z0-9]{20,}"), "Kimi / Moonshot API key"),
    (re.compile(r"\bsk-[A-Za-z0-9]{32,}\b"), "OpenAI-style API key"),
    (re.compile(r"AKIA[0-9A-Z]{16}"), "AWS access key id"),
    (re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"), "private key"),
    (re.compile(r"\bghp_[A-Za-z0-9]{36}\b"), "GitHub personal access token"),
]


def tracked_files() -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return [ROOT / name for name in result.stdout.split("\0") if name]


def main() -> int:
    failures: list[str] = []
    for path in tracked_files():
        if not path.is_file():
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        for pattern, label in PATTERNS:
            for match in pattern.finditer(text):
                line = text.count("\n", 0, match.start()) + 1
                failures.append(f"{path.relative_to(ROOT)}:{line}: looks like a {label}")

    if failures:
        print("Possible credentials in tracked files:", file=sys.stderr)
        print("\n".join(failures), file=sys.stderr)
        print(
            "\nMove the value into an ignored file (see .env.example) and rotate it.",
            file=sys.stderr,
        )
        return 1

    print("No credentials found in tracked files.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
