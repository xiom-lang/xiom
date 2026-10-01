// XIOM -- Selfhost lexer (Phase 1: parity port of crates/xiom-lexer)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Port of `crates/xiom-lexer/src/lib.rs` to XIOM. Control flow, spans, and
// error text mirror the Rust lexer 1:1:
//   * char-indexed scanning with an independent UTF-8 BYTE accumulator
//     (exactly like Rust's `byte` field -- `parse_numeric_suffix` restores
//     only `pos`, so an invalid suffix leaves `byte`/`col` advanced; token
//     text is built char-wise, never by slicing source bytes),
//   * all leading U+FEFF BOMs stripped byte-wise before decoding,
//   * Unicode whitespace skipping (Rust `char::is_whitespace` set),
//   * comment/shebang skipping without trivia collection (the dump mode
//     needs only the token stream),
//   * `\xNN` = codepoint U+00NN, `\u{...}` with U+FFFD fallback,
//   * big-integer classification via u128 range (textual 39-decimal /
//     32-hex digit checks after leading-zero stripping).
//
// TokenKind variants are `Tk`-prefixed: bare keyword-like names collide with
// builtin variants (`Some`/`None`), literals (`True`/`False`) and catalog
// symbols (`Pipe` is `xiom.os.Pipe`). `lx_kind_tag` maps them back to the
// Rust names for the dump.
//
// Internal helpers are FREE functions taking `&mut Lexer` / `&Lexer` rather
// than receiver methods: a nested `self` method call that mutates a receiver
// scalar field does NOT propagate the mutation in the current compiler
// (repro tmp/sprintc/phase1_lexer/probe8.xi; probe9.xi is the working
// shape). The `lx_` prefix mirrors runtime_ffi's `rt_` convention.
//
// `dump_tokens` emits the canonical token dump consumed by the Phase 1 gate
// (crates/xiom-codegen/tests/full_diff_tests.rs::diff_tokens). Format:
//
//   {line}:{col}:{byte_start}:{byte_end} {TAG}[ {PAYLOAD}]
//
// matching the Rust driver's `--dump-tokens` byte for byte (modulo the
// platform line terminator: XIOM's `io.print` is CRT printf, so Windows
// pipes see CRLF; the harness compares line-wise).
//
// Float payloads are dumped as the token LEXEME, not the parsed value:
// the selfhost side has neither a correctly-rounded decimal->f64 parser
// nor an i64<->f64 bitcast intrinsic yet (see num.float's TODO); value
// parity is deferred and will surface in the parser/AST phase. Everything
// else is value-exact (Int/BigInt hex, Str/Ident decoded bytes, Error text).
// Str payloads are raw DECODED byte vectors (not `Str`): XIOM `Str` is
// NUL-terminated, so `"\0"` would truncate a Str payload while the Rust
// lexer preserves the NUL. `lexeme` for Str tokens is the raw source text;
// the parser only reads lexemes for Int/BigInt/Float suffixes, so the
// difference is inert.

module selfhost_lexer

use xiom.io;
use xiom.string;
use xiom.char;
use selfhost_runtime_ffi;

// ============================================================================
// Token definitions (mirror of xiom_lexer::TokenKind)
// ============================================================================

pub type TokenKind = enum {
  // Keywords
  TkLet, TkVar, TkConst, TkFn, TkReturn,
  TkBreak, TkContinue,
  TkIf, TkElif, TkElse, TkMatch, TkWhile, TkFor, TkIn,
  TkSpawn, TkAwait, TkComptime, TkAsm, TkDefer,
  TkMove,
  TkModule, TkUse, TkPub, TkAs,
  TkType, TkEnum, TkInterface, TkDerive, TkImpl,
  TkTrue, TkFalse, TkSelf,
  TkSome, TkNone, TkOkV, TkErrV,
  TkUnsafe, TkExtern, TkIs,
  // Literals
  TkIdent(name: Str),
  TkInt(v: UInt),
  TkBigInt(hi: UInt, lo: UInt),
  TkFloat(v: Float64),
  TkStr(bytes: Vec[UInt8]),
  TkChar(cp: Int),
  // Operators / punctuation
  TkDot, TkComma, TkSemicolon, TkColon, TkColonColon,
  TkLParen, TkRParen, TkLBrace, TkRBrace, TkLBracket, TkRBracket,
  TkAt, TkArrow, TkFatArrow, TkQuestion,
  TkPlus, TkMinus, TkStar, TkSlash, TkPercent, TkCaret, TkTilde,
  TkBang, TkAmp, TkPipe, TkAmpersand,
  TkEq, TkEqEq, TkNeq, TkLt, TkGt, TkLe, TkGe,
  TkAndAnd, TkOrOr,
  TkPlusEq, TkMinusEq, TkStarEq, TkSlashEq, TkPercentEq,
  TkDotDot, TkDotDotEq,
  TkUnderscore, TkHash,
  // Special
  TkEof,
  TkError(msg: Str),
}

pub type Token = {
  kind: TokenKind;
  line: Int;
  col: Int;
  byte_start: Int;
  byte_end: Int;
  lexeme: Str;
}

// ============================================================================
// Lexer state
// ============================================================================

pub type Lexer = {
  src: Str;
  /// Decoded codepoints (Rust `Vec<char>`).
  chars: Vec[Int];
  /// Char index.
  pos: Int;
  line: Int;
  col: Int;
  /// Independent BYTE accumulator (post-BOM), Rust `byte` parity.
  byte: Int;
}

pub fn Lexer.new(src_in: Str) -> Lexer {
  // BUG 23 #6 parity: every leading UTF-8 BOM is stripped. Byte offsets in
  // spans are relative to the POST-BOM source.
  let n0 = src_in.len();
  var b0 = 0;
  var bom = true;
  while bom {
    if b0 + 3 <= n0
      && string.byte_at(src_in, b0) == 239
      && string.byte_at(src_in, b0 + 1) == 187
      && string.byte_at(src_in, b0 + 2) == 191
    {
      b0 = b0 + 3;
    } else {
      bom = false;
    }
  }
  let src = string.str_slice(src_in, b0, n0);
  var chars = Vec[Int].new();
  let n = src.len();
  var i = 0;
  while i < n {
    let lead = string.byte_at(src, i) as Int;
    var l = 1;
    if lead >= 240 { l = 4; }
    elif lead >= 224 { l = 3; }
    elif lead >= 192 { l = 2; }
    if i + l > n { l = n - i; }
    chars.push(selfhost_runtime_ffi.rt_char_at(src, i));
    i = i + l;
  }
  return Lexer{
    src: src,
    chars: chars,
    pos: 0,
    line: 1,
    col: 1,
    byte: 0
  };
}

// ============================================================================
// Character predicates (Rust parities)
// ============================================================================

fn lx_is_digit(c: Int) -> Bool {
  return c >= 48 && c <= 57;
}

fn lx_is_alpha(c: Int) -> Bool {
  return (c >= 65 && c <= 90) || (c >= 97 && c <= 122);
}

fn lx_is_ident_char(c: Int) -> Bool {
  return lx_is_digit(c) || lx_is_alpha(c) || c == 95;
}

fn lx_is_hex_digit(c: Int) -> Bool {
  return lx_is_digit(c) || (c >= 65 && c <= 70) || (c >= 97 && c <= 102);
}

/// Rust `char::is_whitespace` (Unicode White_Space property).
fn lx_is_ws(cp: Int) -> Bool {
  if cp == 32 || (cp >= 9 && cp <= 13) { return true; }
  if cp == 133 || cp == 160 || cp == 5760 { return true; }
  if cp >= 8192 && cp <= 8202 { return true; }
  if cp == 8232 || cp == 8233 || cp == 8239 || cp == 8287 || cp == 12288 { return true; }
  return false;
}

/// UTF-8 length of a codepoint (Rust `char::len_utf8`).
fn lx_cp_len(cp: Int) -> Int {
  if cp < 128 { return 1; }
  if cp < 2048 { return 2; }
  if cp < 65536 { return 3; }
  return 4;
}

// ============================================================================
// Lexer primitives (free functions: see the header note)
// ============================================================================

fn lx_peek(lx: &Lexer) -> Int {
  if lx.pos >= lx.chars.len() { return -1; }
  return lx.chars[lx.pos];
}

fn lx_peek_n(lx: &Lexer, k: Int) -> Int {
  if lx.pos + k >= lx.chars.len() { return -1; }
  return lx.chars[lx.pos + k];
}

fn lx_advance(lx: &mut Lexer) -> Int {
  if lx.pos >= lx.chars.len() { return -1; }
  let c = lx.chars[lx.pos];
  lx.pos = lx.pos + 1;
  lx.byte = lx.byte + lx_cp_len(c);
  if c == 10 {
    lx.line = lx.line + 1;
    lx.col = 1;
  } else {
    lx.col = lx.col + 1;
  }
  return c;
}

/// Build a token whose byte span runs from `sb` to the CURRENT byte offset.
fn lx_mk(lx: &Lexer, kind: TokenKind, sl: Int, sc: Int, sb: Int, lexeme: Str) -> Token {
  return Token{
    kind: kind,
    line: sl,
    col: sc,
    byte_start: sb,
    byte_end: lx.byte,
    lexeme: lexeme
  };
}

fn lx_error_at(lx: &Lexer, sl: Int, sc: Int, sb: Int, msg: Str) -> Token {
  return lx_mk(lx, TkError(msg), sl, sc, sb, "");
}

fn lx_eof(lx: &Lexer) -> Token {
  return lx_mk(lx, TkEof, lx.line, lx.col, lx.byte, "");
}

// --- advance_while specializations (Rust builds Strings char-wise) ---------

fn lx_push_ascii(buf: &mut Vec[UInt8], c: Int) {
  buf.push(c as UInt8);
}

fn lx_aw_digit(lx: &mut Lexer) -> Str {
  var buf = Vec[UInt8].new();
  var go = true;
  while go {
    let c = lx_peek(lx);
    if lx_is_digit(c) {
      lx_push_ascii(buf, c);
      lx_advance(lx);
    } else {
      go = false;
    }
  }
  return Str::from_utf8(buf);
}

fn lx_aw_ident(lx: &mut Lexer) -> Str {
  var buf = Vec[UInt8].new();
  var go = true;
  while go {
    let c = lx_peek(lx);
    if lx_is_ident_char(c) {
      lx_push_ascii(buf, c);
      lx_advance(lx);
    } else {
      go = false;
    }
  }
  return Str::from_utf8(buf);
}

fn lx_aw_hex(lx: &mut Lexer) -> Str {
  var buf = Vec[UInt8].new();
  var go = true;
  while go {
    let c = lx_peek(lx);
    if lx_is_hex_digit(c) || c == 95 {
      lx_push_ascii(buf, c);
      lx_advance(lx);
    } else {
      go = false;
    }
  }
  return Str::from_utf8(buf);
}

fn lx_aw_hex_digits(lx: &mut Lexer) -> Str {
  var buf = Vec[UInt8].new();
  var go = true;
  while go {
    let c = lx_peek(lx);
    if lx_is_hex_digit(c) {
      lx_push_ascii(buf, c);
      lx_advance(lx);
    } else {
      go = false;
    }
  }
  return Str::from_utf8(buf);
}

fn lx_aw_int_part(lx: &mut Lexer) -> Str {
  var buf = Vec[UInt8].new();
  var go = true;
  while go {
    let c = lx_peek(lx);
    if lx_is_digit(c) || c == 95 {
      lx_push_ascii(buf, c);
      lx_advance(lx);
    } else {
      go = false;
    }
  }
  return Str::from_utf8(buf);
}

fn lx_aw_to_newline(lx: &mut Lexer) {
  var go = true;
  while go {
    let c = lx_peek(lx);
    if c >= 0 && c != 10 { lx_advance(lx); } else { go = false; }
  }
}

// --- small string helpers ---------------------------------------------------

/// UTF-8 encoding of one codepoint as a Str.
fn lx_cp_str(cp: Int) -> Str {
  var buf = Vec[UInt8].new();
  char.encode_utf8(to_char(cp), buf);
  return Str::from_utf8(buf);
}

fn lx_remove_underscores(s: Str) -> Str {
  var buf = Vec[UInt8].new();
  var i = 0;
  let n = s.len();
  while i < n {
    let b = string.byte_at(s, i);
    if b != 95 { buf.push(b); }
    i = i + 1;
  }
  return Str::from_utf8(buf);
}

fn lx_has_underscore(s: Str) -> Bool {
  var i = 0;
  let n = s.len();
  while i < n {
    if string.byte_at(s, i) == 95 { return true; }
    i = i + 1;
  }
  return false;
}

fn lx_strip_leading_zeros(s: Str) -> Str {
  var k = 0;
  let n = s.len();
  while k < n && string.byte_at(s, k) == 48 { k = k + 1; }
  return string.str_slice(s, k, n);
}

fn lx_hex_val(b: UInt8) -> Int {
  let c = b as Int;
  if c >= 48 && c <= 57 { return c - 48; }
  if c >= 97 && c <= 102 { return c - 87; }
  return c - 55;
}

/// Rust `char::to_digit(16).unwrap_or(0)` over an advance() result (-1 = EOF
/// maps to '0' -> 0, mirroring `unwrap_or('0')`).
fn lx_hex_digit_or0(c: Int) -> Int {
  if c >= 48 && c <= 57 { return c - 48; }
  if c >= 65 && c <= 70 { return c - 55; }
  if c >= 97 && c <= 102 { return c - 87; }
  return 0;
}

fn lx_last_byte_is_digit(s: Str) -> Bool {
  if s.len() == 0 { return false; }
  return lx_is_digit(string.byte_at(s, s.len() - 1) as Int);
}

fn lx_parse_hex_u32(digits: Str) -> Int {
  var v = 0;
  var i = 0;
  let n = digits.len();
  while i < n {
    v = v * 16 + lx_hex_val(string.byte_at(digits, i));
    i = i + 1;
  }
  return v;
}

fn lx_parse_dec_u128(digits: Str) -> UInt128 {
  var v: UInt128 = 0 as UInt128;
  var i = 0;
  let n = digits.len();
  while i < n {
    v = v * (10 as UInt128) + ((string.byte_at(digits, i) as Int - 48) as UInt128);
    i = i + 1;
  }
  return v;
}

fn lx_parse_hex_u128(digits: Str) -> UInt128 {
  var v: UInt128 = 0 as UInt128;
  var i = 0;
  let n = digits.len();
  while i < n {
    v = v * (16 as UInt128) + (lx_hex_val(string.byte_at(digits, i)) as UInt128);
    i = i + 1;
  }
  return v;
}

fn lx_u128_is_big(v: UInt128) -> Bool {
  return v > (18446744073709551615 as UInt128);
}

fn lx_bigint_hi(v: UInt128) -> UInt {
  return (v >> 64) as UInt;
}

fn lx_bigint_lo(v: UInt128) -> UInt {
  return (v & (18446744073709551615 as UInt128)) as UInt;
}

// ============================================================================
// Numeric literal suffix (Rust parse_numeric_suffix parity)
// ============================================================================

fn lx_parse_numeric_suffix(lx: &mut Lexer) -> Str {
  let saved = lx.pos;
  var prefix = "";
  let c = lx_peek(lx);
  if c == 105 || c == 117 {
    lx_advance(lx);
    if c == 105 { prefix = "i"; } else { prefix = "u"; }
  } elif c == 102 {
    lx_advance(lx);
    prefix = "f";
  } else {
    return "";
  }
  let width = lx_aw_digit(lx);
  let suffix = prefix + width;
  if suffix == "i8" || suffix == "i16" || suffix == "i32" || suffix == "i64"
    || suffix == "u8" || suffix == "u16" || suffix == "u32" || suffix == "u64"
    || suffix == "f32" || suffix == "f64"
  {
    return suffix;
  }
  // Rust quirk: only the char index is restored; `byte`/`col` stay advanced.
  lx.pos = saved;
  return "";
}

// ============================================================================
// Number scanning
// ============================================================================

fn lx_number(lx: &mut Lexer, sl: Int, sc: Int, sb: Int) -> Token {
  let c = lx_peek(lx);
  if c == 48 && (lx_peek_n(lx, 1) == 120 || lx_peek_n(lx, 1) == 88) {
    return lx_hex_number(lx, sl, sc, sb);
  }
  let int_part = lx_aw_int_part(lx);
  if lx_peek(lx) == 46 && lx_is_digit(lx_peek_n(lx, 1)) {
    return lx_decimal_float(lx, int_part, sl, sc, sb);
  }
  if lx_peek(lx) == 101 || lx_peek(lx) == 69 {
    return lx_integer_exponent(lx, int_part, sl, sc, sb);
  }
  return lx_decimal_int(lx, int_part, sl, sc, sb);
}

fn lx_exponent_text(lx: &mut Lexer) -> Str {
  var exp = lx_cp_str(lx_advance(lx));
  if lx_peek(lx) == 43 || lx_peek(lx) == 45 { exp = exp + lx_cp_str(lx_advance(lx)); }
  return exp + lx_aw_digit(lx);
}

fn lx_decimal_float(lx: &mut Lexer, int_part: Str, sl: Int, sc: Int, sb: Int) -> Token {
  lx_advance(lx); // .
  let frac = lx_aw_digit(lx);
  var exp = "";
  if lx_peek(lx) == 101 || lx_peek(lx) == 69 {
    exp = lx_exponent_text(lx);
  }
  let full = int_part + "." + frac + exp;
  if lx_has_underscore(int_part) || (exp.len() > 0 && !lx_last_byte_is_digit(exp)) {
    return lx_mk(lx, TkError("malformed float literal " + full), sl, sc, sb, full);
  }
  match string.str_to_float(full) {
    Ok(f) => {
      let suffix = lx_parse_numeric_suffix(lx);
      return lx_mk(lx, TkFloat(f), sl, sc, sb, full + suffix);
    }
    Err(_) => {
      return lx_mk(lx, TkError("malformed float literal " + full), sl, sc, sb, full);
    }
  }
}

fn lx_integer_exponent(lx: &mut Lexer, int_part: Str, sl: Int, sc: Int, sb: Int) -> Token {
  let exp = int_part + lx_exponent_text(lx);
  if lx_has_underscore(int_part) || !lx_last_byte_is_digit(exp) {
    return lx_mk(lx, TkError("malformed float literal " + exp), sl, sc, sb, exp);
  }
  match string.str_to_float(exp) {
    Ok(f) => {
      let suffix = lx_parse_numeric_suffix(lx);
      return lx_mk(lx, TkFloat(f), sl, sc, sb, exp + suffix);
    }
    Err(_) => {
      return lx_mk(lx, TkError("malformed float literal " + exp), sl, sc, sb, exp);
    }
  }
}

fn lx_decimal_int(lx: &mut Lexer, int_part: Str, sl: Int, sc: Int, sb: Int) -> Token {
  let clean = lx_remove_underscores(int_part);
  let digits = lx_strip_leading_zeros(clean);
  var over = false;
  if clean.len() == 0 {
    over = true;
  } elif digits.len() > 39 {
    over = true;
  } elif digits.len() == 39 && string.str_compare(digits, "340282366920938463463374607431768211455") > 0 {
    over = true;
  }
  if over {
    return lx_mk(
      lx,
      TkError("integer literal " + int_part + " does not fit in 128 bits"),
      sl, sc, sb, int_part
    );
  }
  let v = lx_parse_dec_u128(digits);
  let suffix = lx_parse_numeric_suffix(lx);
  let lexeme = int_part + suffix;
  if lx_u128_is_big(v) {
    return lx_mk(lx, TkBigInt(lx_bigint_hi(v), lx_bigint_lo(v)), sl, sc, sb, lexeme);
  }
  return lx_mk(lx, TkInt(v as UInt), sl, sc, sb, lexeme);
}

fn lx_hex_number(lx: &mut Lexer, sl: Int, sc: Int, sb: Int) -> Token {
  lx_advance(lx); // 0
  lx_advance(lx); // x / X
  let hex = lx_aw_hex(lx);
  let clean = lx_remove_underscores(hex);
  let digits = lx_strip_leading_zeros(clean);
  if clean.len() == 0 || digits.len() > 32 {
    return lx_mk(
      lx,
      TkError("hex integer literal 0x" + hex + " does not fit in 128 bits"),
      sl, sc, sb, "0x" + hex
    );
  }
  let v = lx_parse_hex_u128(digits);
  let suffix = lx_parse_numeric_suffix(lx);
  let lexeme = "0x" + hex + suffix;
  if lx_u128_is_big(v) {
    return lx_mk(lx, TkBigInt(lx_bigint_hi(v), lx_bigint_lo(v)), sl, sc, sb, lexeme);
  }
  return lx_mk(lx, TkInt(v as UInt), sl, sc, sb, lexeme);
}

// ============================================================================
// String and char literals
// ============================================================================

fn lx_push_cp(buf: &mut Vec[UInt8], cp: Int) {
  char.encode_utf8(to_char(cp), buf);
}

fn lx_bytes_copy_str(bytes: Vec[UInt8]) -> Str {
  var copy = Vec[UInt8].new();
  var i = 0;
  while i < bytes.len() {
    copy.push(bytes[i]);
    i = i + 1;
  }
  return Str::from_utf8(copy);
}

fn lx_string_lit(lx: &mut Lexer, sl: Int, sc: Int, sb: Int) -> Token {
  lx_advance(lx); // opening "
  var bytes = Vec[UInt8].new();
  var done = false;
  while !done {
    let c = lx_peek(lx);
    if c < 0 {
      return lx_error_at(lx, sl, sc, sb, "unterminated string literal");
    }
    if c == 34 {
      lx_advance(lx);
      done = true;
    } elif c == 92 {
      lx_advance(lx);
      let e = lx_advance(lx);
      if e == 110 { bytes.push(10 as UInt8); }
      elif e == 116 { bytes.push(9 as UInt8); }
      elif e == 114 { bytes.push(13 as UInt8); }
      elif e == 92 { bytes.push(92 as UInt8); }
      elif e == 34 { bytes.push(34 as UInt8); }
      elif e == 39 { bytes.push(39 as UInt8); }
      elif e == 48 { bytes.push(0 as UInt8); }
      elif e == 98 { bytes.push(8 as UInt8); }
      elif e == 102 { bytes.push(12 as UInt8); }
      elif e == 120 {
        let d1 = lx_hex_digit_or0(lx_advance(lx));
        let d2 = lx_hex_digit_or0(lx_advance(lx));
        lx_push_cp(bytes, (d1 << 4) | d2);
      }
      elif e == 117 {
        if lx_advance(lx) != 123 {
          return lx_error_at(lx, sl, sc, sb, "expected '{' after \\u");
        }
        let hx = lx_aw_hex_digits(lx);
        if lx_advance(lx) != 125 {
          return lx_error_at(lx, sl, sc, sb, "expected '}' after \\u hex digits");
        }
        var cp = 0xFFFD;
        let hd = lx_strip_leading_zeros(hx);
        if hd.len() > 0 && hd.len() <= 8 {
          cp = lx_parse_hex_u32(hd);
          if cp > 1114111 || (cp >= 55296 && cp <= 57343) {
            cp = 0xFFFD;
          }
        }
        lx_push_cp(bytes, cp);
      } else {
        return lx_error_at(lx, sl, sc, sb, "invalid escape sequence");
      }
    } else {
      lx_advance(lx);
      lx_push_cp(bytes, c);
    }
  }
  // Payload: decoded bytes (NUL-preserving). Lexeme: Rust stores the decoded
  // text re-wrapped in quotes; XIOM `Str` cannot hold an embedded NUL, so the
  // lexeme copy may truncate at the first NUL -- inert for the parser, which
  // reads lexemes only for Int/BigInt/Float suffixes.
  let lex = "\"" + lx_bytes_copy_str(bytes) + "\"";
  return lx_mk(lx, TkStr(bytes), sl, sc, sb, lex);
}

fn lx_char_lit(lx: &mut Lexer, sl: Int, sc: Int, sb: Int) -> Token {
  lx_advance(lx); // opening '
  var cp = 0;
  let first = lx_advance(lx);
  if first == 92 {
    let e = lx_advance(lx);
    if e == 110 { cp = 10; }
    elif e == 116 { cp = 9; }
    elif e == 114 { cp = 13; }
    elif e == 92 { cp = 92; }
    elif e == 39 { cp = 39; }
    elif e == 34 { cp = 34; }
    elif e == 48 { cp = 0; }
    elif e == 98 { cp = 8; }
    elif e == 102 { cp = 12; }
    elif e == 120 {
      let d1 = lx_hex_digit_or0(lx_advance(lx));
      let d2 = lx_hex_digit_or0(lx_advance(lx));
      cp = (d1 << 4) | d2;
    } else {
      return lx_error_at(lx, sl, sc, sb, "invalid escape in char literal");
    }
  } elif first >= 0 && first != 39 {
    cp = first;
  } else {
    return lx_error_at(lx, sl, sc, sb, "empty char literal");
  }
  if lx_advance(lx) != 39 {
    return lx_error_at(lx, sl, sc, sb, "unterminated char literal");
  }
  return lx_mk(lx, TkChar(cp), sl, sc, sb, "'" + lx_cp_str(cp) + "'");
}

// ============================================================================
// Identifiers / keywords
// ============================================================================

fn lx_keyword_or_ident(s: Str) -> TokenKind {
  if s == "let" { return TkLet; }
  if s == "var" { return TkVar; }
  if s == "const" { return TkConst; }
  if s == "fn" { return TkFn; }
  if s == "return" { return TkReturn; }
  if s == "break" { return TkBreak; }
  if s == "continue" { return TkContinue; }
  if s == "if" { return TkIf; }
  if s == "elif" { return TkElif; }
  if s == "else" { return TkElse; }
  if s == "match" { return TkMatch; }
  if s == "while" { return TkWhile; }
  if s == "for" { return TkFor; }
  if s == "in" { return TkIn; }
  if s == "spawn" { return TkSpawn; }
  if s == "await" { return TkAwait; }
  if s == "comptime" { return TkComptime; }
  if s == "asm" { return TkAsm; }
  if s == "defer" { return TkDefer; }
  if s == "move" { return TkMove; }
  if s == "module" { return TkModule; }
  if s == "use" { return TkUse; }
  if s == "and" { return TkAndAnd; }
  if s == "or" { return TkOrOr; }
  if s == "not" { return TkBang; }
  if s == "pub" { return TkPub; }
  if s == "as" { return TkAs; }
  if s == "type" { return TkType; }
  if s == "enum" { return TkEnum; }
  if s == "interface" { return TkInterface; }
  if s == "derive" { return TkDerive; }
  if s == "impl" { return TkImpl; }
  if s == "true" { return TkTrue; }
  if s == "false" { return TkFalse; }
  if s == "self" { return TkSelf; }
  if s == "Some" { return TkSome; }
  if s == "None" { return TkNone; }
  if s == "Ok" { return TkOkV; }
  if s == "Err" { return TkErrV; }
  if s == "unsafe" { return TkUnsafe; }
  if s == "extern" { return TkExtern; }
  if s == "is" { return TkIs; }
  if s == "_" { return TkUnderscore; }
  return TkIdent(s);
}

// ============================================================================
// Main scanner
// ============================================================================

fn lx_next_token(lx: &mut Lexer) -> Token {
  // Skip whitespace and comments (trivia not collected).
  var skipping = true;
  while skipping {
    let c = lx_peek(lx);
    if c < 0 {
      skipping = false;
    } elif lx_is_ws(c) {
      lx_advance(lx);
    } elif c == 47 && lx_peek_n(lx, 1) == 47 {
      lx_aw_to_newline(lx);
    } elif c == 47 && lx_peek_n(lx, 1) == 42 {
      let sl = lx.line;
      let sc = lx.col;
      let sb = lx.byte;
      lx_advance(lx);
      lx_advance(lx);
      var closed = false;
      while !closed {
        if lx_peek(lx) == 42 && lx_peek_n(lx, 1) == 47 {
          lx_advance(lx);
          lx_advance(lx);
          closed = true;
        } else {
          let a = lx_advance(lx);
          if a < 0 {
            return lx_error_at(lx, sl, sc, sb, "unterminated block comment");
          }
        }
      }
    } else {
      skipping = false;
    }
  }

  let sl = lx.line;
  let sc = lx.col;
  let sb = lx.byte;
  let ch = lx_peek(lx);
  if ch < 0 {
    return lx_eof(lx);
  }

  // Identifiers and keywords.
  if lx_is_alpha(ch) || ch == 95 {
    let ident = lx_aw_ident(lx);
    return lx_mk(lx, lx_keyword_or_ident(ident), sl, sc, sb, ident);
  }

  // Numbers.
  if lx_is_digit(ch) {
    return lx_number(lx, sl, sc, sb);
  }

  // Strings and chars.
  if ch == 34 {
    return lx_string_lit(lx, sl, sc, sb);
  }
  if ch == 39 {
    return lx_char_lit(lx, sl, sc, sb);
  }

  // Operators and punctuation.
  if ch == 46 { lx_advance(lx); return lx_mk(lx, TkDot, sl, sc, sb, "."); }
  if ch == 44 { lx_advance(lx); return lx_mk(lx, TkComma, sl, sc, sb, ","); }
  if ch == 59 { lx_advance(lx); return lx_mk(lx, TkSemicolon, sl, sc, sb, ";"); }
  if ch == 58 {
    lx_advance(lx);
    if lx_peek(lx) == 58 {
      lx_advance(lx);
      return lx_mk(lx, TkColonColon, sl, sc, sb, "::");
    }
    return lx_mk(lx, TkColon, sl, sc, sb, ":");
  }
  if ch == 40 { lx_advance(lx); return lx_mk(lx, TkLParen, sl, sc, sb, "("); }
  if ch == 41 { lx_advance(lx); return lx_mk(lx, TkRParen, sl, sc, sb, ")"); }
  if ch == 123 { lx_advance(lx); return lx_mk(lx, TkLBrace, sl, sc, sb, "{"); }
  if ch == 125 { lx_advance(lx); return lx_mk(lx, TkRBrace, sl, sc, sb, "}"); }
  if ch == 91 { lx_advance(lx); return lx_mk(lx, TkLBracket, sl, sc, sb, "["); }
  if ch == 93 { lx_advance(lx); return lx_mk(lx, TkRBracket, sl, sc, sb, "]"); }
  if ch == 64 { lx_advance(lx); return lx_mk(lx, TkAt, sl, sc, sb, "@"); }
  if ch == 35 { lx_advance(lx); return lx_mk(lx, TkHash, sl, sc, sb, "#"); }
  if ch == 63 { lx_advance(lx); return lx_mk(lx, TkQuestion, sl, sc, sb, "?"); }
  if ch == 43 {
    lx_advance(lx);
    if lx_peek(lx) == 61 { lx_advance(lx); return lx_mk(lx, TkPlusEq, sl, sc, sb, "+="); }
    return lx_mk(lx, TkPlus, sl, sc, sb, "+");
  }
  if ch == 45 {
    lx_advance(lx);
    if lx_peek(lx) == 62 { lx_advance(lx); return lx_mk(lx, TkArrow, sl, sc, sb, "->"); }
    if lx_peek(lx) == 61 { lx_advance(lx); return lx_mk(lx, TkMinusEq, sl, sc, sb, "-="); }
    return lx_mk(lx, TkMinus, sl, sc, sb, "-");
  }
  if ch == 42 {
    lx_advance(lx);
    if lx_peek(lx) == 61 { lx_advance(lx); return lx_mk(lx, TkStarEq, sl, sc, sb, "*="); }
    return lx_mk(lx, TkStar, sl, sc, sb, "*");
  }
  if ch == 47 {
    lx_advance(lx);
    if lx_peek(lx) == 61 { lx_advance(lx); return lx_mk(lx, TkSlashEq, sl, sc, sb, "/="); }
    return lx_mk(lx, TkSlash, sl, sc, sb, "/");
  }
  if ch == 37 {
    lx_advance(lx);
    if lx_peek(lx) == 61 { lx_advance(lx); return lx_mk(lx, TkPercentEq, sl, sc, sb, "%="); }
    return lx_mk(lx, TkPercent, sl, sc, sb, "%");
  }
  if ch == 94 { lx_advance(lx); return lx_mk(lx, TkCaret, sl, sc, sb, "^"); }
  if ch == 126 { lx_advance(lx); return lx_mk(lx, TkTilde, sl, sc, sb, "~"); }
  if ch == 33 {
    lx_advance(lx);
    if lx_peek(lx) == 61 { lx_advance(lx); return lx_mk(lx, TkNeq, sl, sc, sb, "!="); }
    return lx_mk(lx, TkBang, sl, sc, sb, "!");
  }
  if ch == 38 {
    lx_advance(lx);
    if lx_peek(lx) == 38 { lx_advance(lx); return lx_mk(lx, TkAndAnd, sl, sc, sb, "&&"); }
    return lx_mk(lx, TkAmpersand, sl, sc, sb, "&");
  }
  if ch == 124 {
    lx_advance(lx);
    if lx_peek(lx) == 124 { lx_advance(lx); return lx_mk(lx, TkOrOr, sl, sc, sb, "||"); }
    return lx_mk(lx, TkPipe, sl, sc, sb, "|");
  }
  if ch == 61 {
    lx_advance(lx);
    if lx_peek(lx) == 61 { lx_advance(lx); return lx_mk(lx, TkEqEq, sl, sc, sb, "=="); }
    if lx_peek(lx) == 62 { lx_advance(lx); return lx_mk(lx, TkFatArrow, sl, sc, sb, "=>"); }
    return lx_mk(lx, TkEq, sl, sc, sb, "=");
  }
  if ch == 60 {
    lx_advance(lx);
    if lx_peek(lx) == 61 { lx_advance(lx); return lx_mk(lx, TkLe, sl, sc, sb, "<="); }
    return lx_mk(lx, TkLt, sl, sc, sb, "<");
  }
  if ch == 62 {
    lx_advance(lx);
    if lx_peek(lx) == 61 { lx_advance(lx); return lx_mk(lx, TkGe, sl, sc, sb, ">="); }
    return lx_mk(lx, TkGt, sl, sc, sb, ">");
  }

  lx_advance(lx);
  let t = lx_cp_str(ch);
  return lx_mk(lx, TkError("unexpected character: '" + t + "'"), sl, sc, sb, t);
}

// ============================================================================
// Tokenization
// ============================================================================

fn lx_kind_is_eof(k: TokenKind) -> Bool {
  match k {
    TkEof => { return true; }
    _ => { return false; }
  }
}

pub fn lx_tokenize(lx: &mut Lexer) -> Vec[Token] {
  // M10 shebang parity: skip `#!...` on line 1 (as trivia in Rust); the
  // line/col counters are re-synced while `byte` keeps the consumed bytes.
  if lx.pos == 0 && lx_peek(lx) == 35 && lx_peek_n(lx, 1) == 33 {
    lx_aw_to_newline(lx);
    if lx_peek(lx) == 10 { lx_advance(lx); }
    lx.line = 1;
    lx.col = 1;
  }
  var toks = Vec[Token].new();
  var done = false;
  while !done {
    let t = lx_next_token(lx);
    let is_eof = lx_kind_is_eof(t.kind);
    toks.push(t);
    if is_eof { done = true; }
  }
  return toks;
}

/// Phase 0 compatibility: token count.
pub fn lex_count(src: Str) -> Int {
  var lx = Lexer.new(src);
  let ts = lx_tokenize(&mut lx);
  return ts.len();
}

// ============================================================================
// Canonical dump (Phase 1 parity gate)
// ============================================================================

fn lx_nibble_byte(n: Int) -> UInt8 {
  if n < 10 { return (48 + n) as UInt8; }
  return (87 + n) as UInt8;
}

fn lx_push_byte(buf: &mut Vec[UInt8], b: UInt8) {
  buf.push(b);
}

fn lx_push_str(buf: &mut Vec[UInt8], s: Str) {
  var i = 0;
  let n = s.len();
  while i < n {
    buf.push(string.byte_at(s, i));
    i = i + 1;
  }
}

fn lx_push_dec(buf: &mut Vec[UInt8], v: Int) {
  if v <= 0 {
    buf.push(48 as UInt8);
    return;
  }
  var digits: [20]UInt8;
  var pos = 20;
  var n = v;
  while n > 0 {
    pos = pos - 1;
    digits[pos] = lx_nibble_byte(n % 10);
    n = n / 10;
  }
  while pos < 20 {
    buf.push(digits[pos]);
    pos = pos + 1;
  }
}

fn lx_push_hex_str(buf: &mut Vec[UInt8], s: Str) {
  var i = 0;
  let n = s.len();
  while i < n {
    let b = string.byte_at(s, i) as Int;
    buf.push(lx_nibble_byte(b >> 4));
    buf.push(lx_nibble_byte(b & 15));
    i = i + 1;
  }
}

fn lx_push_hex_vec(buf: &mut Vec[UInt8], bytes: Vec[UInt8]) {
  var i = 0;
  while i < bytes.len() {
    let b = bytes[i] as Int;
    buf.push(lx_nibble_byte(b >> 4));
    buf.push(lx_nibble_byte(b & 15));
    i = i + 1;
  }
}

fn lx_push_hex_int(buf: &mut Vec[UInt8], v: Int) {
  if v <= 0 {
    buf.push(48 as UInt8);
    return;
  }
  var tmp: [16]UInt8;
  var pos = 16;
  var x = v;
  while x > 0 {
    pos = pos - 1;
    tmp[pos] = lx_nibble_byte(x & 15);
    x = x >> 4;
  }
  while pos < 16 {
    buf.push(tmp[pos]);
    pos = pos + 1;
  }
}

fn lx_push_u64_hex(buf: &mut Vec[UInt8], v: UInt) {
  var k = 15;
  while k >= 0 {
    let n = ((v >> (4 * k)) & (15 as UInt)) as Int;
    buf.push(lx_nibble_byte(n));
    k = k - 1;
  }
}

fn lx_kind_tag(k: TokenKind) -> Str {
  match k {
    TkLet => { return "Let"; }
    TkVar => { return "Var"; }
    TkConst => { return "Const"; }
    TkFn => { return "Fn"; }
    TkReturn => { return "Return"; }
    TkBreak => { return "Break"; }
    TkContinue => { return "Continue"; }
    TkIf => { return "If"; }
    TkElif => { return "Elif"; }
    TkElse => { return "Else"; }
    TkMatch => { return "Match"; }
    TkWhile => { return "While"; }
    TkFor => { return "For"; }
    TkIn => { return "In"; }
    TkSpawn => { return "Spawn"; }
    TkAwait => { return "Await"; }
    TkComptime => { return "Comptime"; }
    TkAsm => { return "Asm"; }
    TkDefer => { return "Defer"; }
    TkMove => { return "Move"; }
    TkModule => { return "Module"; }
    TkUse => { return "Use"; }
    TkPub => { return "Pub"; }
    TkAs => { return "As"; }
    TkType => { return "Type"; }
    TkEnum => { return "Enum"; }
    TkInterface => { return "Interface"; }
    TkDerive => { return "Derive"; }
    TkImpl => { return "Impl"; }
    TkTrue => { return "True"; }
    TkFalse => { return "False"; }
    TkSelf => { return "Self_"; }
    TkSome => { return "Some"; }
    TkNone => { return "None"; }
    TkOkV => { return "Ok_"; }
    TkErrV => { return "Err_"; }
    TkUnsafe => { return "Unsafe"; }
    TkExtern => { return "Extern"; }
    TkIs => { return "Is"; }
    TkDot => { return "Dot"; }
    TkComma => { return "Comma"; }
    TkSemicolon => { return "Semicolon"; }
    TkColon => { return "Colon"; }
    TkColonColon => { return "ColonColon"; }
    TkLParen => { return "LParen"; }
    TkRParen => { return "RParen"; }
    TkLBrace => { return "LBrace"; }
    TkRBrace => { return "RBrace"; }
    TkLBracket => { return "LBracket"; }
    TkRBracket => { return "RBracket"; }
    TkAt => { return "At"; }
    TkArrow => { return "Arrow"; }
    TkFatArrow => { return "FatArrow"; }
    TkQuestion => { return "Question"; }
    TkPlus => { return "Plus"; }
    TkMinus => { return "Minus"; }
    TkStar => { return "Star"; }
    TkSlash => { return "Slash"; }
    TkPercent => { return "Percent"; }
    TkCaret => { return "Caret"; }
    TkTilde => { return "Tilde"; }
    TkBang => { return "Bang"; }
    TkAmp => { return "Amp"; }
    TkPipe => { return "Pipe"; }
    TkAmpersand => { return "Ampersand"; }
    TkEq => { return "Eq"; }
    TkEqEq => { return "EqEq"; }
    TkNeq => { return "Neq"; }
    TkLt => { return "Lt"; }
    TkGt => { return "Gt"; }
    TkLe => { return "Le"; }
    TkGe => { return "Ge"; }
    TkAndAnd => { return "AndAnd"; }
    TkOrOr => { return "OrOr"; }
    TkPlusEq => { return "PlusEq"; }
    TkMinusEq => { return "MinusEq"; }
    TkStarEq => { return "StarEq"; }
    TkSlashEq => { return "SlashEq"; }
    TkPercentEq => { return "PercentEq"; }
    TkDotDot => { return "DotDot"; }
    TkDotDotEq => { return "DotDotEq"; }
    TkUnderscore => { return "Underscore"; }
    TkHash => { return "Hash"; }
    TkEof => { return "Eof"; }
    _ => { return "?"; }
  }
}

fn lx_dump_kind(buf: &mut Vec[UInt8], t: Token) {
  let lex = t.lexeme;
  match t.kind {
    TkIdent(name) => {
      lx_push_str(buf, "Ident ");
      lx_push_hex_str(buf, name);
    }
    TkInt(v) => {
      lx_push_str(buf, "Int ");
      lx_push_u64_hex(buf, v);
    }
    TkBigInt(hi, lo) => {
      lx_push_str(buf, "BigInt ");
      lx_push_u64_hex(buf, hi);
      lx_push_u64_hex(buf, lo);
    }
    TkFloat(v) => {
      lx_push_str(buf, "Float ");
      lx_push_hex_str(buf, lex);
    }
    TkStr(bytes) => {
      lx_push_str(buf, "Str ");
      lx_push_hex_vec(buf, bytes);
    }
    TkChar(cp) => {
      lx_push_str(buf, "Char ");
      lx_push_hex_int(buf, cp);
    }
    TkError(msg) => {
      lx_push_str(buf, "Error ");
      lx_push_hex_str(buf, msg);
    }
    _ => {
      lx_push_str(buf, lx_kind_tag(t.kind));
    }
  }
}

pub fn dump_tokens(src: Str) -> Int {
  var lx = Lexer.new(src);
  let toks = lx_tokenize(&mut lx);
  var out = Vec[UInt8].new();
  var i = 0;
  while i < toks.len() {
    let t = toks[i];
    lx_push_dec(out, t.line);
    lx_push_byte(out, 58);
    lx_push_dec(out, t.col);
    lx_push_byte(out, 58);
    lx_push_dec(out, t.byte_start);
    lx_push_byte(out, 58);
    lx_push_dec(out, t.byte_end);
    lx_push_byte(out, 32);
    lx_dump_kind(out, t);
    lx_push_byte(out, 10);
    i = i + 1;
  }
  io.print(Str::from_utf8(out));
  return 0;
}
