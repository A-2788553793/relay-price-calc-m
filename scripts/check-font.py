#!/usr/bin/env python3
"""Offline font integrity and source-literal coverage check (Python stdlib only).

Run: python3 scripts/check-font.py
Source commit: f8d157532fbfaeda587e826d4cd5b21a49186f7c
Font source:
https://raw.githubusercontent.com/notofonts/noto-cjk/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf
License source:
https://raw.githubusercontent.com/notofonts/noto-cjk/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/LICENSE
Unmodified static Regular OTF: 16,437,364 bytes; SIL OFL 1.1.
Bundling/embedding permitted; retain LICENSE and font copyright metadata.
This is a cmap check, not a rendering test or arbitrary-user-input guarantee.
The lightweight literal scanner handles quoted/raw strings, character literals,
Unicode escapes and nested comments; it is not a full Rust/Slint parser.
"""

import hashlib
from pathlib import Path
import re
import struct
import sys
import unicodedata

ROOT = Path(__file__).resolve().parents[1]
FONT = "assets/fonts/NotoSansCJKsc-Regular.otf"
LICENSE = "assets/fonts/LICENSE"
HASHES = {
    FONT: "2c76254f6fc379fddfce0a7e84fb5385bb135d3e399294f6eeb6680d0365b74b",
    LICENSE: "6a73f9541c2de74158c0e7cf6b0a58ef774f5a780bf191f2d7ec9cc53efe2bf2",
}
FAMILY = "Noto Sans CJK SC"
TOKEN = re.compile(
    r'//[^\n]*|/\*|r(?P<hashes>\#*)"(?P<raw>.*?)"(?P=hashes)'
    r'|"(?P<string>(?:\\.|[^"\\])*)"'
    r"|'(?P<char>\\u\{[0-9a-fA-F_]+\}|\\.|[^'\\\n])'",
    re.DOTALL,
)
ESCAPE = re.compile(r'\\u\{([0-9a-fA-F_]+)\}|\\u([0-9a-fA-F]{4})|\\U([0-9a-fA-F]{8})|\\x([0-9a-fA-F]{2})|\\(.)', re.DOTALL)


def u16(data, pos):
    return struct.unpack_from(">H", data, pos)[0]


def u32(data, pos):
    return struct.unpack_from(">I", data, pos)[0]


def literals(text):
    pos = 0
    while True:
        match = TOKEN.search(text, pos)
        if match is None:
            return
        pos = match.end()
        if match.group() == "/*":
            depth = 1
            while depth:
                boundary = re.search(r'/\*|\*/', text[pos:])
                if boundary is None:
                    raise ValueError("Unterminated block comment")
                depth += 1 if boundary.group() == "/*" else -1
                pos += boundary.end()
            continue
        if match.group().startswith("//"):
            continue
        if match.group("raw") is not None:
            yield match.start(), match.group("raw")
            continue
        value = match.group("string")
        if value is None:
            value = match.group("char")
        def decode(escape):
            for group in escape.groups()[:4]:
                if group is not None:
                    return chr(int(group.replace("_", ""), 16))
            return {"n": "\n", "r": "\r", "t": "\t", "0": "\0"}.get(escape.group(5), escape.group(5))
        yield match.start(), ESCAPE.sub(decode, value)


def font_metadata(data):
    if data[:4] != b"OTTO":
        raise ValueError("Expected single static CFF OpenType font")
    tables = {}
    for i in range(u16(data, 4)):
        tag, _, offset, length = struct.unpack_from(">4sIII", data, 12 + i * 16)
        tables[tag] = data[offset:offset + length]
    if b"fvar" in tables or b"CFF " not in tables:
        raise ValueError("Font must be static CFF OTF")
    names = {}
    table = tables[b"name"]
    for i in range(u16(table, 2)):
        platform, _, _, name_id, length, offset = struct.unpack_from(">6H", table, 6 + i * 12)
        if platform in (0, 3):
            start = u16(table, 4) + offset
            names.setdefault(name_id, set()).add(table[start:start + length].decode("utf-16-be"))
    if names.get(1) != {FAMILY} or names.get(2) != {"Regular"}:
        raise ValueError("Unexpected family/subfamily: " + repr(names))
    # The hash-pinned upstream font supplies full Unicode format 12 maps.
    cmap = tables[b"cmap"]
    covered = set()
    for i in range(u16(cmap, 2)):
        platform, encoding, offset = struct.unpack_from(">HHI", cmap, 4 + i * 8)
        if platform != 0 and (platform != 3 or encoding not in (1, 10)):
            continue
        if u16(cmap, offset) != 12:
            continue
        for group in range(u32(cmap, offset + 12)):
            first, last, glyph = struct.unpack_from(">III", cmap, offset + 16 + group * 12)
            covered.update(range(first if glyph else first + 1, last + 1))
    if not covered:
        raise ValueError("Required Unicode cmap format 12 missing")
    return names, covered


def check():
    for path, expected in HASHES.items():
        data = (ROOT / path).read_bytes()
        actual = hashlib.sha256(data).hexdigest()
        if actual != expected:
            raise ValueError(f"SHA-256 mismatch: {path}: expected {expected}, got {actual}")
        print(f"SHA-256 OK {path}: {actual} ({len(data):,} bytes)")
    if (ROOT / "licenses/NotoSansCJK-OFL.txt").read_bytes() != (ROOT / LICENSE).read_bytes():
        raise ValueError("Bundled license copy differs from upstream LICENSE")
    names, covered = font_metadata((ROOT / FONT).read_bytes())
    print(f"Family: {FAMILY}; subfamily: Regular; static CFF OTF")
    print("PostScript: " + ", ".join(sorted(names.get(6, set()))))
    print("Version: " + ", ".join(sorted(names.get(5, set()))))
    required = ("ui/app.slint", "src/model.rs", "src/calc.rs")
    for path in required:
        if not (ROOT / path).is_file():
            raise ValueError("Required source missing: " + path)
    files = sorted((ROOT / "ui").rglob("*.slint")) + sorted((ROOT / "src").rglob("*.rs"))
    origins = {}
    for path in files:
        text = path.read_text(encoding="utf-8")
        for offset, value in literals(text):
            origin = f"{path.relative_to(ROOT)}:{text.count(chr(10), 0, offset) + 1}"
            for char in value:
                if char.isspace() or unicodedata.category(char).startswith("C"):
                    continue
                origins.setdefault(ord(char), set()).add(origin)
    if not origins:
        raise ValueError("No visible literal characters found")
    han = sum("CJK" in unicodedata.name(chr(cp), "") and "IDEOGRAPH" in unicodedata.name(chr(cp), "") for cp in origins)
    print(f"Coverage: {len(files)} source files; {len(origins)} visible literal codepoints; {han} Han codepoints")
    for path in files:
        print("  " + str(path.relative_to(ROOT)))
    missing = sorted(set(origins) - covered)
    for cp in missing:
        print(f"MISSING U+{cp:04X} {chr(cp)!r}: " + ", ".join(sorted(origins[cp])), file=sys.stderr)
    if missing:
        raise ValueError(f"{len(missing)} literal characters missing from font cmap")
    print("PASS: all UI/Rust literal characters covered, including model/calc Chinese strings")


if __name__ == "__main__":
    try:
        check()
    except (OSError, ValueError, KeyError, struct.error) as exc:
        print("FAIL: " + str(exc), file=sys.stderr)
        sys.exit(1)
