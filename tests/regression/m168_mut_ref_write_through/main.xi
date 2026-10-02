// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m168 lock (packages commit bad44b2): assignment to a `&mut T` parameter
// must write THROUGH the address it carries. Before the fix the assignment
// rebound the local slot (`store i64* inttoptr(99), i64**`) and the write
// was silently dropped (xiom.svm's shuffle seed never advanced).
module m168_mut_ref_write_through

use xiom.io;

fn set99(s: &mut Int) {
  s = 99;
}

fn reset(v: &mut Vec[Int]) {
  v = Vec[Int].new();
}

fn setstr(s: &mut Str) {
  s = "changed";
}

fn main() -> Int {
  var st: Int = 10;
  set99(&mut st);
  if st != 99 { return 1; }

  var v = Vec[Int].new();
  v.push(1);
  v.push(2);
  reset(&mut v);
  if v.len() != 0 { return 2; }

  var s: Str = "orig";
  setstr(&mut s);
  if s != "changed" { return 3; }

  io.println("m168 ok");
  return 0;
}
