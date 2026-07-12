#!/usr/bin/env python3
"""Migrate match-arm separators only inside Language-owned Ken source.

Catalog edits are confined to `ken`, `ken-repl`, and `ken-error` fences. Rust
edits are confined to string literals that demonstrably contain Ken match
syntax.
"""

from __future__ import annotations

import argparse
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
KEN_FENCE = re.compile(r"^\s*(`{3,}|~{3,})(ken(?:-repl|-error)?)\b")
MATCH_ARROW = re.compile(r"\bmatch\b[\s\S]*(?:⇒|=>)")


def replace_arrows(text: str) -> tuple[str, int, int]:
    unicode_count = text.count("⇒")
    ascii_count = text.count("=>")
    return text.replace("⇒", "↦").replace("=>", "|->"), unicode_count, ascii_count


def migrate_catalog(text: str) -> tuple[str, int, int]:
    out: list[str] = []
    fence: str | None = None
    unicode_count = 0
    ascii_count = 0

    for line in text.splitlines(keepends=True):
        if fence is None:
            opened = KEN_FENCE.match(line)
            if opened:
                fence = opened.group(1)
            out.append(line)
            continue

        if re.match(rf"^\s*{re.escape(fence)}\s*$", line.rstrip("\r\n")):
            fence = None
            out.append(line)
            continue

        line, unicode_delta, ascii_delta = replace_arrows(line)
        unicode_count += unicode_delta
        ascii_count += ascii_delta
        out.append(line)

    if fence is not None:
        raise ValueError("unterminated Ken fence")
    return "".join(out), unicode_count, ascii_count


def raw_string_end(text: str, start: int) -> int | None:
    if text[start] != "r":
        return None
    pos = start + 1
    while pos < len(text) and text[pos] == "#":
        pos += 1
    if pos >= len(text) or text[pos] != '"':
        return None
    hashes = text[start + 1 : pos]
    end = text.find('"' + hashes, pos + 1)
    if end < 0:
        raise ValueError("unterminated Rust raw string")
    return end + 1 + len(hashes)


def quoted_string_end(text: str, start: int) -> int:
    pos = start + 1
    while pos < len(text):
        if text[pos] == "\\":
            pos += 2
        elif text[pos] == '"':
            return pos + 1
        else:
            pos += 1
    raise ValueError("unterminated Rust string")


def migrate_rust(text: str) -> tuple[str, int, int]:
    out: list[str] = []
    pos = 0
    unicode_count = 0
    ascii_count = 0

    while pos < len(text):
        if text.startswith("//", pos):
            end = text.find("\n", pos)
            end = len(text) if end < 0 else end
            out.append(text[pos:end])
            pos = end
            continue
        if text.startswith("/*", pos):
            end = text.find("*/", pos + 2)
            if end < 0:
                raise ValueError("unterminated Rust block comment")
            end += 2
            out.append(text[pos:end])
            pos = end
            continue

        raw_end = raw_string_end(text, pos)
        if raw_end is not None:
            end = raw_end
        elif text[pos] == '"':
            end = quoted_string_end(text, pos)
        else:
            out.append(text[pos])
            pos += 1
            continue

        literal = text[pos:end]
        prefix = text[max(0, pos - 32) : pos].rstrip()
        is_expected_output = prefix.endswith("contains(")
        if not is_expected_output and MATCH_ARROW.search(literal):
            literal, unicode_delta, ascii_delta = replace_arrows(literal)
            unicode_count += unicode_delta
            ascii_count += ascii_delta
        out.append(literal)
        pos = end

    return "".join(out), unicode_count, ascii_count


def candidates() -> list[Path]:
    paths = sorted((ROOT / "catalog").glob("**/*.ken.md"))
    paths.extend(
        path
        for path in sorted((ROOT / "crates").glob("**/tests/**/*.rs"))
        if path.name != "match_arm_glyph.rs"
    )
    paths.extend(sorted((ROOT / "crates/ken-elaborator/src").glob("**/*.rs")))
    return paths


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()

    total_unicode = 0
    total_ascii = 0
    changed = 0
    for path in candidates():
        text = path.read_text()
        if path.suffixes[-2:] == [".ken", ".md"]:
            migrated, unicode_count, ascii_count = migrate_catalog(text)
        else:
            migrated, unicode_count, ascii_count = migrate_rust(text)
        if not (unicode_count or ascii_count):
            continue
        changed += 1
        total_unicode += unicode_count
        total_ascii += ascii_count
        print(f"{path.relative_to(ROOT)}\t⇒={unicode_count}\t=>={ascii_count}")
        if args.write:
            path.write_text(migrated)

    action = "migrated" if args.write else "found"
    print(
        f"{action}: files={changed} ⇒={total_unicode} =>={total_ascii} "
        f"total={total_unicode + total_ascii}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
