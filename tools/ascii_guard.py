#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
# SPDX-License-Identifier: MIT OR Apache-2.0

"""
ascii_guard.py - detect and repair mojibake / non-ASCII in the XIOM codebase.

Why this exists
---------------
The repo repeatedly accumulated "mojibake": typographic Unicode characters
(em dash U+2014, curly quotes U+201C/U+201D, curly apostrophes U+2018/U+2019,
arrows U+2192) that were round-tripped through Windows-1252 and stored as
garbage (byte sequences like `C3 83 C2 A2 C3 A2 E2 82 AC ...`). The policy is:

    * All tracked text files must be pure ASCII (bytes 0x00-0x7F).
    * Exceptions are explicit and few (e.g. UTF-8 test data in
      examples/stdlib_smoke/smoke_convert_utf.xi).

Usage
-----
    python tools/ascii_guard.py check [--repo PATH] [--exclude GLOB ...]
        Exit 1 if any tracked text file contains non-ASCII bytes.
        Used by CI (see .github/workflows/ci.yml) and the pre-commit hook.

    python tools/ascii_guard.py repair [--apply|--dry-run] [--repo PATH]
        Decode mojibake back to its original characters, then transliterate
        every remaining non-ASCII character to ASCII. --dry-run only reports.

Repair algorithm
----------------
1. Tokenize maximal runs of "marker" characters (everything in U+0080-U+00FF
   plus the CP1252-inherited punctuation). Legit UTF-8 (e.g. accented text
   like "hello" with a U+00E9) keeps its bytes because the round-trip
   (cp1252 encode -> utf-8 decode) only succeeds for byte sequences that
   form valid UTF-8 after CP1252 encoding.
2. Round-trip each token up to 6 times (mojibake is level-1, level-2, ...).
3. Transliterate whatever non-ASCII remains: NFKD accent-stripping, a
   punctuation map (em dash -> "--", curly quotes -> straight, arrow -> "->"),
   then a last-resort '?' that is ALWAYS reported for manual review.
"""

import argparse
import os
import re
import subprocess
import sys
import unicodedata

# --------------------------------------------------------------------------
# Configuration
# --------------------------------------------------------------------------

# Directories / files that are allowed to contain non-ASCII (tracked only).
# registry/ mirrors npm package metadata (legit Unicode), node_modules is
# third-party data. The remaining entries are files whose Unicode is
# load-bearing functionality or intentional test data: Chinese/Korean numeral
# conversion, emoji/CJK/Cyrillic string-handling smoke tests, fuzz inputs,
# playground lesson UI content, and the installer ASCII-art logo.
EXCLUDED_PREFIXES = (
    "registry/",
    "node_modules/",
    "examples/stdlib_smoke/smoke_convert_utf.xi",
    "examples/stdlib_smoke/smoke_text2.xi",
    "examples/stdlib_smoke/smoke_string_emoji.xi",
    "examples/stdlib_smoke/smoke_string_ea_width.xi",
    "examples/stdlib_smoke/smoke_string_unicode.xi",
    "examples/stdlib_smoke/smoke_encoding_punycode.xi",
    "crates/xiom-codegen/tests/fuzz_tests.rs",
    "tests/regression/m36_e16.xi",
    "stdlib/xiom/format/numbering.xi",
    "stdlib/xiom/math/number_systems.xi",
    "xiom-playground/lessons/",
    "xiom-playground/js/lessons.js",
    "xiom-playground/index.html",
    "install.ps1",
)

# Markers that can appear inside mojibake. U+0080-U+00FF covers the Latin-1
# view (U+00C3, U+00E2, U+00A9, ...); the rest are the CP1252-inherited
# chars -- including the continuation-byte views (U+0192, U+2026, U+2013,
# U+2014, U+02C6, U+02DC, U+0160, U+0161, U+017D, U+017E) that appear in
# level-2+ mojibake. A missing marker splits the run and breaks reversal.
MARKER_RE = re.compile(
    "[\u0080-\u00ff\u0192\u0160\u0161\u017d\u017e\u2012-\u201f"
    "\u2020-\u2022\u2025\u2026\u2030\u2039\u203a\u20ac\u2122"
    "\u0152\u0153\u0178\u02c6\u02dc]+"
)

# Mojibake blocks are maximal runs of marker characters: ASCII never appears
# inside a run (characters like the U+2019 in a deep-mojibake run are
# markers, not ASCII apostrophes). Never include ASCII in the round-tripped
# unit -- a legit U+2014 or U+20AC anywhere in a large unit makes its whole
# decode fail.
RUN_RE = MARKER_RE

# Punctuation / symbol transliteration applied AFTER accent folding.
PUNCT_MAP = {
    "\u2019": "'", "\u2018": "'", "\u201a": "'", "\u201b": "'",
    "\u201c": '"', "\u201d": '"', "\u201e": '"', "\u201f": '"',
    "\u2039": "<", "\u203a": ">",
    "\u2014": "--", "\u2013": "-", "\u2015": "--", "\u2012": "-",
    "\u2026": "...",
    "\u2192": "->", "\u2190": "<-", "\u2194": "<->",
    "\u2191": "^", "\u2193": "v", "\u21d2": "=>", "\u21a6": "->",
    "\u00d7": "x", "\u00f7": "/",
    "\u00a9": "(c)", "\u00ae": "(R)", "\u2122": "(TM)",
    "\u00b1": "+/-", "\u00b0": "deg", "\u00bd": "1/2", "\u00bc": "1/4",
    "\u00be": "3/4", "\u00b2": "^2", "\u00b3": "^3",
    "\u00a0": " ", "\u2009": " ", "\u200a": " ", "\u202f": " ",
    "\ufeff": "",  # BOM
    # Isolated mojibake-alphabet leftovers (from earlier partial fixes).
    # These map to a neutral ASCII equivalent, never silently to a letter.
    "\u0080": "", "\u0081": "", "\u0082": "", "\u0083": "", "\u0084": "",
    "\u0085": "", "\u0086": "", "\u0087": "", "\u0088": "", "\u0089": "",
    "\u008a": "", "\u008b": "", "\u008c": "", "\u008d": "", "\u008e": "",
    "\u008f": "", "\u0090": "", "\u0091": "", "\u0092": "", "\u0093": "",
    "\u0094": "", "\u0095": "", "\u0096": "", "\u0097": "", "\u0098": "",
    "\u0099": "", "\u009a": "", "\u009b": "", "\u009c": "", "\u009d": "",
    "\u009e": "", "\u009f": "",
    "\u20ac": "EUR", "\u00a2": "c", "\u00a3": "GBP", "\u00a5": "JPY",
    "\u0192": "f", "\u00a4": "$", "\u00a6": "|", "\u00a7": "S",
    "\u00a8": '"', "\u00b5": "u", "\u00b6": "P", "\u00aa": "a",
    "\u00ba": "o", "\u00bf": "?", "\u00a1": "!", "\u00ac": "not",
    "\u00ab": "<<", "\u00bb": ">>",
    "\u2020": "*", "\u2021": "*", "\u2030": "%", "\u2022": "-",
    "\u00b7": "-", "\u00c6": "AE", "\u00e6": "ae", "\u00d8": "O",
    "\u00f8": "o", "\u00d0": "D", "\u00f0": "d", "\u00de": "Th",
    "\u00fe": "th", "\u00df": "ss", "\u0152": "OE", "\u0153": "oe",
    "\u0178": "Y", "\u02c6": "^",
    "\u25ba": ">", "\u2713": "[OK]", "\u2717": "[FAIL]",
    "\u27e6": "[[", "\u27e7": "]]",
    # Mathematical operators / Greek used in prose.
    "\u2264": "<=", "\u2265": ">=", "\u2260": "!=", "\u2248": "~=",
    "\u2261": "==", "\u221a": "sqrt", "\u222b": "int", "\u2211": "sum",
    "\u2212": "-", "\u221e": "inf", "\u2208": "in", "\u2218": "o",
    "\u2227": "^", "\u2229": "&", "\u222a": "|",
    "\u2286": "subset", "\u2287": "superset", "\u208b": "-",
    "\u0391": "Alpha", "\u03ae": "eta", "\u03bd": "nu",
    "\u03a6": "Phi", "\u03c6": "phi", "\u03a9": "Omega",
    "\u03a3": "Sigma", "\u03bb": "lambda",
    "\u03b1": "alpha", "\u03b2": "beta", "\u03bc": "mu", "\u03c0": "pi",
    "\u03c4": "tau", "\u0394": "Delta", "\u03b4": "delta",
    "\u03b3": "gamma", "\u03b8": "theta",
    # Box drawing / dingbats (ASCII-art tables).
    "\u2500": "-", "\u2501": "-", "\u2550": "=",
    "\u2502": "|", "\u2503": "|", "\u2551": "|",
    "\u2514": "`", "\u251c": "|", "\u250c": "+", "\u2510": "+",
    "\u2518": "+", "\u2524": "|", "\u2534": "+", "\u252c": "+",
    "\u253c": "+",
    "\u2554": "+", "\u2557": "+", "\u255a": "+", "\u255d": "+",
    "\u25bc": "v", "\u25b2": "^", "\u25a0": "#", "\u25c6": "*",
    "\u25cf": "*", "\u25aa": "-", "\u25a1": "[ ]", "\u25cb": "o",
    "\u25c9": "*", "\u25ce": "o", "\u25c4": "<", "\u25b6": ">",
    "\u25b8": ">", "\u25ba": ">",
    "\u2581": "#", "\u2582": "#", "\u2583": "#", "\u2584": "#",
    "\u2585": "#", "\u2586": "#", "\u2587": "#", "\u2588": "#",
    "\u2591": ".", "\u2592": ".", "\u2593": ".",
    "\u2605": "*", "\u2606": "*", "\u2610": "[ ]", "\u2611": "[X]",
    "\u2699": "[SETTINGS]", "\u26d4": "[STOP]", "\u2764": "[HEART]",
    "\u21ba": "[RELOAD]", "\u2011": "-", "\u2b50": "[STAR]",
    "\u2016": "||", "\u207b": "-", "\u1e9e": "SS", "\u03c3": "sigma",
    "\u0400": "Ye", "\u0401": "Yo", "\u0450": "ye", "\u0451": "yo",
    # Emoji status markers used in docs. NOTE: chars above U+FFFF need the
    # \U0001xxxx escape form (8 hex digits), \u only takes 4.
    "\u2705": "[OK]", "\u2714": "[OK]", "\u274c": "[FAIL]",
    "\u26a0": "[WARN]", "\u23f3": "[WIP]",
    "\U0001f6a7": "[WIP]", "\U0001f527": "[TOOLING]",
    "\U0001f389": "[DONE]", "\u2603": "[SNOW]",
    "\U0001f4e6": "[PKG]", "\U0001f916": "[ROBOT]",
    "\U0001f3c6": "[TROPHY]", "\U0001f7e0": "[ORANGE]",
    "\U0001f7e1": "[YELLOW]", "\U0001f7e2": "[GREEN]",
    "\U0001f534": "[RED]", "\U0001f535": "[BLUE]",
    "\u26aa": "[WHITE]", "\U0001f532": "[NO]", "\U0001f504": "[REFRESH]",
    "\U0001f4ca": "[CHART]", "\U0001f4dc": "[SCROLL]",
    "\U0001f4cb": "[CLIPBOARD]", "\U0001f44b": "[HI]",
    "\U0001f680": "[ROCKET]", "\u26a1": "[BOLT]",
    "\u2b1c": "[ ]",
    "\ufe0f": "",  # variation selector
    "\ufffd": "?",  # replacement char: real data loss, always reported
}

# Byte-extension allowlist for text files (everything else is treated as
# binary and skipped). Extensionless files are allowed when their basename
# matches a known text name.
TEXT_EXTENSIONS = {
    ".rs", ".xi", ".c", ".h", ".cpp", ".hpp", ".cc", ".py", ".md", ".txt",
    ".json", ".toml", ".yaml", ".yml", ".sh", ".ps1", ".js", ".ts", ".html",
    ".css", ".ll", ".x", ".ini", ".cfg", ".mk", ".cmake", ".inc", ".S", ".s",
    ".zig", ".go", ".cs", ".jsx", ".tsx", ".vue", ".sql", ".proto", ".lock",
    ".gitignore", ".dockerignore", ".editorconfig", ".gitattributes",
    ".cargo", ".bat", ".cmake",
}
TEXT_NAMES = {
    "makefile", "license", "readme", "dockerfile", "cargo.toml",
    "rustfmt.toml", "clippy.toml", ".gitignore", ".gitattributes",
    ".editorconfig", ".dockerignore",
}


# --------------------------------------------------------------------------
# File discovery
# --------------------------------------------------------------------------

def is_text_file(relpath):
    """True for paths we are willing to scan/rewrite."""
    base = os.path.basename(relpath)
    ext = os.path.splitext(base)[1].lower()
    if ext in TEXT_EXTENSIONS:
        return True
    if not ext and base.lower() in TEXT_NAMES:
        return True
    return False


def tracked_files(repo):
    """Yield (relpath, abspath) for every git-tracked text file."""
    out = subprocess.run(
        ["git", "ls-files", "-z"], cwd=repo, capture_output=True, check=True
    )
    for rel in out.stdout.split(b"\0"):
        if not rel:
            continue
        relpath = rel.decode("utf-8", "replace").replace("\\", "/")
        if not is_text_file(relpath):
            continue
        if relpath.startswith(EXCLUDED_PREFIXES):
            continue
        abspath = os.path.join(repo, relpath)
        yield relpath, abspath


def staged_files(repo):
    """Yield (relpath, abspath) for staged text files (pre-commit hook)."""
    out = subprocess.run(
        ["git", "diff", "--cached", "--name-only", "-z"],
        cwd=repo, capture_output=True, check=True
    )
    for rel in out.stdout.split(b"\0"):
        if not rel:
            continue
        relpath = rel.decode("utf-8", "replace").replace("\\", "/")
        if not is_text_file(relpath):
            continue
        if relpath.startswith(EXCLUDED_PREFIXES):
            continue
        abspath = os.path.join(repo, relpath)
        yield relpath, abspath


def load_bytes(abspath):
    try:
        with open(abspath, "rb") as fh:
            return fh.read()
    except OSError as exc:
        print("  [skip] unreadable: {} ({})".format(abspath, exc))
        return None


# --------------------------------------------------------------------------
# Mojibake reversal
# --------------------------------------------------------------------------

def try_roundtrip(token):
    """Reverse one mojibake level: bytes -> UTF-8 decode.

    Encoding must mirror the CP1252 "view" of the original UTF-8 bytes:
    chars <= U+00FF encode as their own byte value (this also covers the
    C1 controls U+0080-U+009F, which Python's cp1252 codec refuses to
    encode), while the CP1252-inherited punctuation (EUR, smart quotes,
    trademark, ...) maps through the cp1252 codec. None if either step
    fails.
    """
    try:
        out = bytearray()
        for ch in token:
            o = ord(ch)
            if o <= 0xFF:
                out.append(o)
            else:
                out += ch.encode("cp1252")
        return bytes(out).decode("utf-8")
    except (UnicodeEncodeError, UnicodeDecodeError):
        return None


def _marker_count(s):
    """Total number of marker characters (not runs) in s."""
    return sum(len(m) for m in MARKER_RE.findall(s))


def fix_mojibake(text):
    """Reverse 1..N levels of CP1252 round-trip mojibake.

    Two complementary passes, repeated until stable:
    * line-level: round-trips whole lines. Deep mojibake embeds original
      ASCII (spaces, punctuation) inside the runs; only whole-line round
      trips can reverse those.
    * run-level: round-trips maximal marker runs for everything the
      line-level pass cannot touch (lines that also contain a legit marker
      such as an em dash break the whole-line decode).
    """
    for _ in range(10):
        if not MARKER_RE.search(text):
            break
        before = text
        text = "\n".join(_fix_unit(line) for line in text.split("\n"))
        text = MARKER_RE.sub(_run_repl, text)
        if text == before:
            break
    return text


def _fix_unit(line):
    """Round-trip a whole line (if it contains markers) as far as possible."""
    if not MARKER_RE.search(line):
        return line
    current = line
    for _ in range(8):
        nxt = try_roundtrip(current)
        if nxt is None or nxt == current:
            break
        current = nxt
        if not MARKER_RE.search(current):
            break
    if _marker_count(current) >= _marker_count(line):
        return line
    return current


def _run_repl(match):
    segment = match.group(0)
    current = segment
    for _ in range(8):
        nxt = try_roundtrip(current)
        if nxt is None or nxt == current:
            break
        current = nxt
        if not MARKER_RE.search(current):
            break
    # Only accept the result if it is strictly "less mojibake".
    if _marker_count(current) >= _marker_count(segment):
        return segment
    return current


# --------------------------------------------------------------------------
# Transliteration
# --------------------------------------------------------------------------

_COMBINING = re.compile("[\u0300-\u036f]")


def transliterate(text):
    """ASCII-only version of text. Returns (clean, replaced_with_question)."""
    clean = []
    questioned = set()
    for ch in text:
        if ord(ch) < 0x80:
            clean.append(ch)
            continue
        if ch == "\ufffd":
            # Replacement char: the original bytes were destroyed at some
            # point. Always surface it for manual review.
            clean.append("[U+FFFD]")
            questioned.add("FFFD")
            continue
        folded = _COMBINING.sub("", unicodedata.normalize("NFKD", ch))
        if all(ord(c) < 0x80 for c in folded):
            clean.append(folded)
            continue
        if ch in PUNCT_MAP:
            clean.append(PUNCT_MAP[ch])
            continue
        # Fallback: an explicit, greppable ASCII stand-in so the change is
        # never silent. Always reported for manual review.
        clean.append("[U+{:04X}]".format(ord(ch)))
        questioned.add("{:04X}".format(ord(ch)))
    return "".join(clean), questioned


_CP1252_EXT = {
    0x80: 0x20AC, 0x82: 0x201A, 0x83: 0x0192, 0x84: 0x201E,
    0x85: 0x2026, 0x86: 0x2020, 0x87: 0x2021, 0x88: 0x02C6,
    0x89: 0x2030, 0x8A: 0x0160, 0x8B: 0x2039, 0x8C: 0x0152,
    0x8E: 0x017D, 0x91: 0x2018, 0x92: 0x2019, 0x93: 0x201C,
    0x94: 0x201D, 0x95: 0x2022, 0x96: 0x2013, 0x97: 0x2014,
    0x98: 0x02DC, 0x99: 0x2122, 0x9A: 0x0161, 0x9B: 0x203A,
    0x9C: 0x0153, 0x9E: 0x017E, 0x9F: 0x0178,
}


def decode_cp1252_lenient(data):
    """Decode bytes as Windows-1252; the 5 undefined bytes (0x81, 0x8D,
    0x8F, 0x90, 0x9D) map to their C1 control code points instead of
    raising UnicodeDecodeError."""
    chars = []
    for b in data:
        if 0x80 <= b <= 0x9F:
            chars.append(chr(_CP1252_EXT.get(b, b)))
        else:
            chars.append(chr(b))
    return "".join(chars)


# --------------------------------------------------------------------------
# Repair
# --------------------------------------------------------------------------

def repair_file(relpath, abspath, apply_changes, dry_run):
    data = load_bytes(abspath)
    if data is None:
        return None
    if not any(b >= 0x80 for b in data):
        return None

    # Detect EOL and BOM so rewriting is byte-faithful otherwise. Note the
    # lone-LF count must subtract CRLF pairs (every \r\n contains a \n).
    crlf_count = data.count(b"\r\n")
    lf_only = data.count(b"\n") - crlf_count
    eol = b"\r\n" if crlf_count > lf_only else b"\n"
    has_bom = data.startswith(b"\xef\xbb\xbf")
    body = data[3:] if has_bom else data
    try:
        text = body.decode("utf-8")
    except UnicodeDecodeError:
        # Some historical files were saved as Windows-1252. Decode them
        # that way; the mojibake/transliteration steps then normalize them.
        text = decode_cp1252_lenient(body)

    fixed = fix_mojibake(text)
    clean, questioned = transliterate(fixed)
    # BOMs are non-ASCII and serve no purpose here: always strip them.
    # Strip trailing '\r' before joining: when the source used CRLF, splitting
    # on '\n' leaves '\r' on each line and re-joining would double it.
    new_body = eol.decode("ascii").join(
        line.rstrip("\r") for line in clean.split("\n")).encode("utf-8")

    changed = new_body != data
    if not changed:
        return None

    action = "fixed" if (apply_changes and not dry_run) else "would fix"
    q_count = clean.count("[U+")
    print("  [{}] {} ({} non-ascii chars removed{})".format(
        action, relpath, sum(1 for b in data if b >= 0x80),
        ", {} -> [U+XXXX]".format(q_count) if q_count else ""))
    if questioned:
        print("        ^ UNMAPPED CHARS (became '?'): {} - REVIEW REQUIRED".format(
            ", ".join(sorted(questioned))))
    if apply_changes and not dry_run:
        with open(abspath, "wb") as fh:
            fh.write(new_body)
    return relpath


def cmd_repair(args):
    print("== Repairing mojibake in git-tracked text files ==")
    total = 0
    for relpath, abspath in tracked_files(args.repo):
        if repair_file(relpath, abspath, args.apply, args.dry_run):
            total += 1
    print("== {} file(s) {} ==".format(
        total, "WOULD be changed (dry-run)" if args.dry_run else "changed"))
    if args.dry_run:
        print("Re-run with --apply to write the changes.")


# --------------------------------------------------------------------------
# Check (guard)
# --------------------------------------------------------------------------

def cmd_check(args):
    bad = 0
    files = staged_files(args.repo) if args.staged else tracked_files(args.repo)
    for relpath, abspath in files:
        data = load_bytes(abspath)
        if data is None:
            continue
        if any(b >= 0x80 for b in data):
            bad += 1
            print("NON-ASCII: {}".format(relpath))
    if bad:
        print("FAIL: {} file(s) contain non-ASCII bytes.".format(bad))
        print("Run `python tools/ascii_guard.py repair --apply` to fix.")
        return 1
    print("OK: checked {} file(s); all pure ASCII.".format(
        "staged" if args.staged else "tracked"))
    return 0


# --------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    p_check = sub.add_parser("check", help="fail on non-ASCII tracked files")
    p_check.add_argument("--repo", default=".", help="repo root (default: cwd)")
    p_check.add_argument("--staged", action="store_true",
                         help="only check staged files (pre-commit hook)")

    p_repair = sub.add_parser("repair", help="decode mojibake + ASCII-ize")
    p_repair.add_argument("--repo", default=".", help="repo root (default: cwd)")
    p_repair.add_argument("--apply", action="store_true",
                          help="write changes (default: dry-run)")
    p_repair.add_argument("--dry-run", action="store_true",
                          help="report only (default)")

    args = parser.parse_args()
    args.repo = os.path.abspath(args.repo)
    if args.command == "check":
        sys.exit(cmd_check(args))
    elif args.command == "repair":
        sys.exit(cmd_repair(args))


if __name__ == "__main__":
    main()
