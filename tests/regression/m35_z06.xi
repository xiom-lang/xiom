// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z06: extern_C+unsafe+cast+generic+contract+Result+match+module+compound_assign+while
type Handle = { id: Int; len: Int; }
enum MemOp { Alloc, Free, Resize }
extern "C" { fn abs(c: Int) -> Int; }
fn safe_abs(v: Int) -> Result[Int, Str]
  requires: v >= -1000
{
  var r: Int = 0;
  unsafe { r = abs(v); }
  return Ok(r);
}
fn mem_ops[T](op: MemOp, raw: Handle) -> Handle
  requires: raw.len >= 0
{
  match op {
    Alloc => { var sz = raw.len; sz += 8; return Handle{ id: raw.id; len: sz; }; }
    Free => { return Handle{ id: raw.id; len: 0; }; }
    Resize => { return Handle{ id: raw.id; len: raw.len * 2; }; }
  }
}
module sys {
  pub fn do_abs(v: Int) -> Result[Int, Str] { return safe_abs(v); }
  pub fn alloc(op: MemOp, h: Handle) -> Handle { return mem_ops(op, h); }
}
use sys.do_abs;
use sys.alloc;
fn main() -> Int {
  var v: Int = -42;
  var h = Handle{ id: 1; len: 16; };
  var r1 = do_abs(v);
  var h2 = alloc(MemOp.Resize, h);
  var h3 = alloc(MemOp.Free, h2);
  var i = 0; var acc = 0;
  while i < 5 { acc += 1; i += 1; }
  match r1 {
    Ok(r) => { if r == 42 && h3.len == 0 && acc == 5 { return 0; } }
    Err(_) => { return 1; }
  }
  return 2;
}
