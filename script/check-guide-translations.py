#!/usr/bin/env python3
"""Check guide source/navigation parity, local links, anchors and literal examples."""
from pathlib import Path
import re
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1] / "docs/guide"


def unfenced(text):
    return re.sub(r"(?ms)^([ \t]*)```[^\n]*\n.*?^\1```[^\n]*$", "", text)


def fences(text):
    return re.findall(r"(?ms)^([ \t]*)```([^\n]*)\n(.*?)^\1```[^\n]*$", text)


def anchors(text):
    used = set()
    for title in re.findall(r"(?m)^#{1,6} (.+)$", unfenced(text)):
        title = title.replace("`", "").replace("*", "").lower()
        base = "".join(c for c in title if c.isalnum() or c in "_ -")
        base = re.sub(r"\s", "-", base)
        anchor = base
        count = 1
        while anchor in used:
            anchor = f"{base}-{count}"
            count += 1
        used.add(anchor)
    return used


def paragraph_counts(text):
    counts = [0]
    for block in re.split(r"\n\s*\n", unfenced(text)):
        first = block.strip()
        if re.match(r"^#{1,6} ", first):
            counts.append(0)
        elif first and not re.match(r"^(?:[-*+] |[0-9]+\. |>|\|)", first):
            counts[-1] += 1
    return counts


def check():
    english = {p.relative_to(ROOT) for p in ROOT.rglob("*.md") if "zh-CN" not in p.parts}
    chinese = {p.relative_to(ROOT / "zh-CN") for p in (ROOT / "zh-CN").rglob("*.md")}
    assert english == chinese, f"Source parity: {english ^ chinese}"
    nav = lambda text: re.findall(r"^  - \[.*?\]\(([^)]+)\)$", text, re.M)
    assert nav((ROOT / "README.md").read_text(encoding="utf-8")) == nav((ROOT / "zh-CN/README.md").read_text(encoding="utf-8")), "Navigation order differs"
    for source in sorted(english):
        en = (ROOT / source).read_text(encoding="utf-8")
        path = ROOT / "zh-CN" / source
        zh = path.read_text(encoding="utf-8")
        plain = unfenced(zh)
        assert paragraph_counts(en) == paragraph_counts(zh), f"Paragraph parity by heading: {path}"
        assert len(re.findall(r"^# ", plain, re.M)) == 1, f"One H1 required: {path}"
        assert re.match(r"# [^\n]+\n\n[^#>\n-]", zh), f"Summary missing: {path}"
        assert [(lang, body) for _, lang, body in fences(en)] == [(lang, body) for _, lang, body in fences(zh)], f"Literal examples changed: {path}"
        for pattern in [r"^#{1,6} ", r"^ *[0-9]+\. ", r"^ *- ", r"^\|", r"^> \[!"]:
            assert len(re.findall(pattern, unfenced(en), re.M)) == len(re.findall(pattern, plain, re.M)), f"Structure {pattern}: {path}"
        for href in re.findall(r"\[[^\]]*\]\(([^)]+)\)", plain):
            url = urlsplit(href)
            if url.scheme or url.netloc or href.startswith("/"):
                continue
            target = (path.parent / unquote(url.path)).resolve() if url.path else path
            assert target.is_file(), f"Missing link: {path}: {href}"
            assert target.is_relative_to(ROOT / "zh-CN"), f"Link leaves Chinese guide: {path}: {href}"
            if url.fragment:
                assert unquote(url.fragment) in anchors(target.read_text(encoding="utf-8")), f"Missing anchor: {path}: {href}"
    print(f"Guide parity, structure, examples and links passed ({len(english)} pages per locale).")


if __name__ == "__main__":
    check()
