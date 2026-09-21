// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z16: closure+generic+extern+unsafe+cast+match+enum+module+contract+compound_assign+derive
type RawBuf = { id: Int; len: Int; } derive[Eq]
enum MemState { Clean, Dirty, Leaked }
extern "C" { fn abs(c: Int) -> Int; }
fn check_buf[T](b: RawBuf) -> MemState
  requires: b.len >= 0
{
  if b.len == 0 { return MemState.Clean; }
  if b.len > 1000 { return MemState.Leaked; }
  return MemState.Dirty;
}
fn dangerous(v: Int) -> Int {
  var r: Int = 0;
  unsafe { r = abs(v); }
  return r;
}
module mem {
  pub fn state(b: RawBuf) -> MemState { return check_buf(b); }
  pub fn danger(v: Int) -> Int { return dangerous(v); }
  pub fn apply(f: fn(Int) -> Int, x: Int) -> Int { return f(x); }
}
use mem.state;
use mem.danger;
use mem.apply;
fn doubler(x: Int) -> Int { return x * 2; }
fn main() -> Int {
  var b1 = RawBuf{ id: 1; len: 0; };
  var b2 = RawBuf{ id: 2; len: 5; };
  var s1 = state(b1);
  var s2 = state(b2);
  var d1 = danger(-10);
  var v = apply(doubler, 7);
  var chk = 0;
  if s1 == MemState.Clean { chk += 1; }
  if s2 == MemState.Dirty { chk += 1; }
  if d1 == 10 { chk += 1; }
  if v == 14 { chk += 1; }
  if chk == 4 { return 0; }
  return 1;
}
