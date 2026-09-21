// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-S11: Error reporting -- error type construction with location and severity
type SourceLoc = { file: Str; line: Int; col: Int; }
type ErrorKind = { code: Int; severity: Int; }
type CompileError = { loc: SourceLoc; kind: ErrorKind; message: Str; }
fn make_loc(file: Str, line: Int, col: Int) -> SourceLoc {
  return SourceLoc{ file: file; line: line; col: col; };
}
fn make_err_kind(code: Int, severity: Int) -> ErrorKind {
  return ErrorKind{ code: code; severity: severity; };
}
fn make_error(loc: SourceLoc, kind: ErrorKind, msg: Str) -> CompileError {
  return CompileError{ loc: loc; kind: kind; message: msg; };
}
fn is_error(e: ErrorKind) -> Bool {
  return e.severity == 0;
}
fn is_warning(e: ErrorKind) -> Bool {
  return e.severity == 1;
}
fn is_note(e: ErrorKind) -> Bool {
  return e.severity == 2;
}
fn error_code(e: CompileError) -> Int {
  return e.kind.code;
}
fn main() -> Int {
  var loc = make_loc("test.xi", 10, 5);
  if loc.line != 10 || loc.col != 5 { return 1; }
  var err_kind = make_err_kind(1001, 0);
  if !is_error(err_kind) { return 2; }
  if is_warning(err_kind) { return 3; }
  var warn_kind = make_err_kind(2001, 1);
  if !is_warning(warn_kind) { return 4; }
  if is_error(warn_kind) { return 5; }
  var note_kind = make_err_kind(3001, 2);
  if !is_note(note_kind) { return 6; }
  var e = make_error(loc, err_kind, "type mismatch");
  if error_code(e) != 1001 { return 7; }
  if e.message.len() == 0 { return 8; }
  return 0;
}
