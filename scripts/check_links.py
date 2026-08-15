"""Check local Markdown links and anchors without accessing the network."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
LINK = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
HEADING = re.compile(r"^#{1,6}\s+(.+?)\s*#*\s*$", re.MULTILINE)


def markdown_files() -> list[Path]:
    files = [ROOT / name for name in ("README.md", "CONTRIBUTING.md", "THIRD_PARTY.md")]
    for directory in (ROOT / "book" / "src", ROOT / "docs", ROOT / "outputs"):
        files.extend(directory.rglob("*.md"))
    return sorted(files)


def slug(value: str) -> str:
    value = re.sub(r"<[^>]+>", "", value)
    value = value.replace("`", "").strip().lower()
    value = re.sub(r"[^\w\- ]", "", value, flags=re.UNICODE)
    return re.sub(r"[ -]+", "-", value).strip("-")


def anchors(path: Path) -> set[str]:
    counts: dict[str, int] = {}
    found: set[str] = set()
    for heading in HEADING.findall(path.read_text(encoding="utf-8")):
        base = slug(heading)
        count = counts.get(base, 0)
        counts[base] = count + 1
        found.add(base if count == 0 else f"{base}-{count}")
    return found


def main() -> int:
    failures: list[str] = []
    for source in markdown_files():
        for raw_target in LINK.findall(source.read_text(encoding="utf-8")):
            target = raw_target.strip().strip("<>")
            parsed = urlsplit(target)
            if parsed.scheme or target.startswith("//"):
                continue
            relative = unquote(parsed.path)
            destination = (source.parent / relative).resolve() if relative else source.resolve()
            if not destination.exists():
                failures.append(f"{source.relative_to(ROOT)}: missing {target}")
                continue
            if parsed.fragment and destination.suffix == ".md":
                wanted = unquote(parsed.fragment)
                if wanted not in anchors(destination):
                    failures.append(
                        f"{source.relative_to(ROOT)}: missing anchor #{wanted} in "
                        f"{destination.relative_to(ROOT)}"
                    )
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    print("Internal Markdown links and anchors are valid.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
