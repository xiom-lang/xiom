// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y14: pointer field struct + unsafe + while + contract + generics + impl + module
type Buffer = { data: *mut Int; cap: Int; len: Int; }
enum WriteKind { Set(v: Int), Add(v: Int), Clear }
fn write[T](buf: Buffer, kind: WriteKind, idx: Int) -> Int
  requires: idx >= 0
  requires: buf.data != (0 as *mut Int)
  ensures: result >= 0
{
  var result = 0;
  match kind {
    Set(v) => {
      unsafe { result = v; }
    }
    Add(v) => {
      var i = 0;
      var acc = 0;
      while i < buf.len { acc = acc + 1; i = i + 1; }
      unsafe { result = acc + v; }
    }
    Clear => {
      var i = 0;
      while i < buf.len { i = i + 1; }
      result = i;
    }
  }
  return result;
}
module buf_mod {
  pub fn do_write(b: Buffer, k: WriteKind, i: Int) -> Int { return write(b, k, i); }
  pub fn cap(b: Buffer) -> Int { return b.cap; }
}
use buf_mod.do_write;
use buf_mod.cap;
fn main() -> Int {
  var val = 0;
  var arr: *mut Int;
  unsafe { arr = &val as *mut Int; }
  var b = Buffer{ data: arr; cap: 8; len: 4; };
  var r1 = do_write(b, WriteKind.Set(99), 0);
  var r2 = do_write(b, WriteKind.Add(10), 0);
  var r3 = do_write(b, WriteKind.Clear, 0);
  if r1 == 99 && r2 == 14 && r3 == 4 { return 0; }
  return 1;
}
