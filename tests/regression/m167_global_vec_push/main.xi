// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m167 lock (packages commit 2d91399): a MODULE-LEVEL `var v: Vec[T]`
// global must take the inline push/read fast paths. Before the fix the
// global fell back to the generic stdlib `Vec.push[T]` body, whose
// hardcoded 8-byte stride and unscaled `data + len` emitted
// `store i8 <handle>, i8*` (clang rejects the IR); float elements also
// read back as their IEEE bit patterns.
module m167_global_vec_push

use xiom.io;
use xiom.convert;

var gs: Vec[Str] = Vec[Str].new();
var gi: Vec[Int] = Vec[Int].new();
var gf: Vec[Float64] = Vec[Float64].new();

fn main() -> Int {
  var i = 0;
  while i < 5 {
    gs.push("s" + int_to_string(i));
    i += 1;
  }
  if gs.len() != 5 { return 1; }
  if gs[0] != "s0" { return 2; }
  if gs[4] != "s4" { return 3; }

  gi.push(10);
  gi.push(20);
  if gi[1] != 20 { return 4; }

  gf.push(1.5);
  gf.push(2.25);
  if gf[0] != 1.5 { return 5; }
  if gf[1] != 2.25 { return 6; }

  io.println(gs.get(2).unwrap());
  io.println("m167 ok");
  return 0;
}
