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
    status = ROOT / "docs/developer/src/status.md"
    if not status.is_file():
        errors.append("docs/developer/src/status.md: missing canonical feature status")
    else:
        content = status.read_text(encoding="utf-8")
        for feature in ("插件安装与卸载", "运行日志", "设置与外观", "打包与公开发行"):
            if f"| {feature} |" not in content:
                errors.append(f"docs/developer/src/status.md: missing feature {feature}")

    # Keep progress claims in one place and prevent old process notes from
    # returning to the maintained design pages.
    design_pages = (
        "README.md", "plan.md", "architecture.md", "compatibility.md",
        "release.md", "ui-parity.md", "ui-pages.md", "ui-assets.md",
        "storage.md", "workflow.md",
    )
    for name in design_pages:
        page = ROOT / "docs/developer/src" / name
        content = page.read_text(encoding="utf-8")
        if "status.md" not in content:
            errors.append(f"{page.relative_to(ROOT)}: missing status reference")
        for phrase in ("子智能体", "状态：已实现", "状态：部分已实现",
                       "状态：计划中", "## 待实现切片"):
            if phrase in content:
                errors.append(f"{page.relative_to(ROOT)}: obsolete phrase {phrase}")
    readme = (ROOT / "README.md").read_text(encoding="utf-8")
    if "docs/developer/src/status.md" not in readme:
        errors.append("README.md: missing canonical feature status link")
    for phrase in ("子智能体", "状态：已实现", "状态：部分已实现",
                   "状态：计划中", "## 待实现切片", "界面复刻", "像素校对"):
        if phrase in readme:
            errors.append(f"README.md: obsolete phrase {phrase}")
    for error in errors:
        print(error, file=sys.stderr)
    if errors:
        return 1
    print("mdBook local links OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
