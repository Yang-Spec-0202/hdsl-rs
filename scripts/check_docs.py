#!/usr/bin/env python3
"""Check local Markdown targets in both mdBook source trees."""

from pathlib import Path
from urllib.parse import unquote
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
LINK = re.compile(r"(?<!!)\[[^]]*\]\(([^)]+)\)")


def slug(value: str) -> str:
    value = value.lower().strip()
    value = re.sub(r"[^\w\-\s]", "", value, flags=re.UNICODE)
    return re.sub(r"\s+", "-", value)


def anchors(path: Path) -> set[str]:
    used: dict[str, int] = {}
    result: set[str] = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        match = re.match(r"^#{1,6}\s+(.+?)\s*#*\s*$", line)
        if match:
            base = slug(match.group(1))
            count = used.get(base, 0)
            used[base] = count + 1
            result.add(base if count == 0 else f"{base}-{count}")
    return result


def check_book(book: Path) -> list[str]:
    source = (book / "src").resolve()
    errors: list[str] = []
    for markdown in source.rglob("*.md"):
        for raw in LINK.findall(markdown.read_text(encoding="utf-8")):
            target = unquote(raw.split(" ", 1)[0].strip("<>"))
            if not target or target.startswith(("https://", "http://", "mailto:")):
                continue
            file_part, _, fragment = target.partition("#")
            dest = (markdown.parent / file_part).resolve() if file_part else markdown.resolve()
            if not dest.is_relative_to(source) or not dest.is_file():
                errors.append(f"{markdown.relative_to(ROOT)}: missing {raw}")
            elif fragment and fragment not in anchors(dest):
                errors.append(f"{markdown.relative_to(ROOT)}: unknown anchor {raw}")
    return errors


def main() -> int:
    errors = check_book(ROOT / "docs/developer") + check_book(ROOT / "docs/user")
    for error in errors:
        print(error, file=sys.stderr)
    if errors:
        return 1
    print("mdBook local links OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
