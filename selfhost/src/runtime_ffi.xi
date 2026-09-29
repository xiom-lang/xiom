// XIOM -- Selfhost runtime helpers (pure XIOM; Phase 0)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Pure-XIOM replacements for the v10-era C helpers in
// stdlib/runtime/xiom_runtime.c (xiom_str_len / xiom_char_at /
// xiom_str_slice / xiom_intern / xiom_lookup / xiom_fn_table_*). The
// selfhost compiler must not depend on C for codegen; this module is its
// only bridge, implemented over the stdlib.
//
// Behavior parity notes (Phase 0):
//   * str_len / str_slice delegate to xiom.string, whose clamping matches
//     the C versions for non-null strings (a NULL Str is not representable
//     in XIOM; the C -1 return is unreachable from safe XIOM callers).
//   * char_at is a direct port of the C UTF-8 codepoint decoder, including
//     its stray-continuation and malformed-sequence behavior (return the
//     leading byte).
//   * The symbol table ports xiom_intern / xiom_lookup: 1-based IDs, 0 =
//     null, deduplicated by content. The Vec scan matches the C table;
//     stdlib Map is itself a linear Vec scan (collections.xi), so no Map
//     mirror is needed.
//
// Deferred (documented, NOT stubbed): float `{:.17e}` formatting belongs
// with the body-emitter port (Phase 5), where its exact bytes are gated by
// T3.
//
// NAMING (2026-09-29): every free function here is `rt_`-prefixed. The
// original reason -- a user module exporting a same-leaf fn (`char_at`)
// poisoned catalog-body checking of unrelated stdlib modules -- is FIXED
// (m162: checker prefers the body's explicit item import; codegen binds the
// recorded target). The prefix is kept for now (it also documents which
// helpers mirror the C runtime surface); rename in the O1 pass if desired.

module selfhost_runtime_ffi

use xiom.io;
use xiom.string;

// === String access =========================================================

/// Byte length. Port of xiom_str_len (non-null domain).
pub fn rt_str_len(src: Str) -> Int {
  return string.str_len(src);
}

/// UTF-8 codepoint at byte `pos`, 0 when out of range. Direct port of
/// xiom_char_at (stdlib/runtime/xiom_runtime.c): decodes 2/3/4-byte
/// sequences, returns the leading byte for stray continuations and
/// malformed sequences.
pub fn rt_char_at(src: Str, pos: Int) -> Int {
  if pos < 0 { return 0; }
  let len = string.str_len(src);
  if pos >= len { return 0; }
  let b = string.byte_at(src, pos) as Int;
  if b < 128 { return b; }
  var seq_len = 0;
  var cp = 0;
  if (b & 224) == 192 { seq_len = 2; cp = b & 31; }
  elif (b & 240) == 224 { seq_len = 3; cp = b & 15; }
  elif (b & 248) == 240 { seq_len = 4; cp = b & 7; }
  else { return b; }
  var i = 1;
  while i < seq_len {
    if pos + i >= len { return b; }
    let cb = string.byte_at(src, pos + i) as Int;
    if (cb & 192) != 128 { return b; }
    cp = (cp << 6) | (cb & 63);
    i = i + 1;
  }
  return cp;
}

/// Substring [start, end) with the C version's clamps. Port of
/// xiom_str_slice.
pub fn rt_str_slice(src: Str, start: Int, end: Int) -> Str {
  return string.str_slice(src, start, end);
}

// === Symbol table (intern / lookup) ========================================

pub type SymbolTable = {
  names: Vec[Str];
}

pub fn rt_symbol_table_new() -> SymbolTable {
  return SymbolTable{ names: Vec[Str].new() };
}

/// Intern `text`: return its 1-based ID, deduplicated by content. Port of
/// xiom_intern (empty text is 0).
pub fn SymbolTable.intern(text: Str) -> Int {
  if text.len() == 0 { return 0; }
  var i = 0;
  while i < names.len() {
    if names[i] == text { return i + 1; }
    i = i + 1;
  }
  names.push(text);
  return names.len();
}

/// Text for a 1-based ID; None for 0 / out-of-range. Port of xiom_lookup.
pub fn SymbolTable.lookup(id: Int) -> Option[Str] {
  if id <= 0 { return None; }
  if id > names.len() { return None; }
  return Some(names[id - 1]);
}

pub fn SymbolTable.count() -> Int {
  return names.len();
}

// === Function table (fn_table_*) ===========================================

pub type FnEntry = {
  name_id: Int;
  ret_type_id: Int;
  param_count: Int;
  body_start: Int;
  body_end: Int;
}

pub type FnTable = {
  entries: Vec[FnEntry];
}

pub fn rt_fn_table_new() -> FnTable {
  return FnTable{ entries: Vec[FnEntry].new() };
}

/// Append one declaration. Port of xiom_fn_table_add.
pub fn FnTable.add(name_id: Int, ret_type_id: Int, param_count: Int, body_start: Int, body_end: Int) {
  entries.push(FnEntry{
    name_id: name_id,
    ret_type_id: ret_type_id,
    param_count: param_count,
    body_start: body_start,
    body_end: body_end
  });
}

/// Port of xiom_fn_table_count.
pub fn FnTable.count() -> Int {
  return entries.len();
}

// === IR buffer (ir_open / ir_header / ir_raw / ir_close) ===================
// Phase 0: a deterministic text builder. The real emitter (Phases 4-6) fills
// it with the Rust emitter's exact line sequence; `emit` flushes it to
// stdout.
//
// NOTE (m163, 2026-09-29): the buffer grows a single Str field instead of a
// Vec[Str] of lines. The Vec-of-lines shape was miscompiled while m163 was
// live (a method field element as a `+` operand lowered to int add +
// inttoptr); m163 is now FIXED (docs/COMPILER_BUGS.md) and the single-field
// shape is kept for Phase 0 simplicity. Restore the Vec-of-lines builder in
// the O1 selfhost code-quality pass (the current shape is O(n^2) in total
// concatenated bytes, fine at Phase 0 scale).

pub type IrBuffer = {
  buf: Str;
}

pub fn rt_ir_open() -> IrBuffer {
  return IrBuffer{ buf: "" };
}

/// Append one output line. Port of xiom_ir_raw's line granularity.
pub fn IrBuffer.raw(text: Str) {
  buf = buf + text + "\n";
}

/// The accumulated text, newline-terminated per line (not printed).
pub fn IrBuffer.text() -> Str {
  return buf;
}

/// Flush the buffer to stdout. Port of xiom_ir_close's flush role.
pub fn IrBuffer.emit() -> Int {
  io.print(buf);
  return 0;
}
