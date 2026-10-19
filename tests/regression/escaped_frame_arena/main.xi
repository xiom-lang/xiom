// Scoped-arena regression for escaped/per-frame Vecs
// (REL-20261019-1343-engine): frame-like structs are returned across
// function boundaries and consumed inside an explicit arena scope.
// xiom_guard_heap_enter/exit are the runtime scoped arena (per-thread);
// Vec/Str/box allocations made by compiled code inside the scope are
// reclaimed wholesale at exit. No arena-allocated value may escape the
// scope (here: everything is consumed before exit).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module escaped_frame_arena

use xiom.io;
use xiom.convert;

extern "C" {
  fn xiom_guard_heap_enter();
  fn xiom_guard_heap_exit();
}

type Frame = { a: Vec[Int]; b: Vec[Float64]; c: Vec[Int]; }

fn make_frame(n: Int) -> Frame {
  var a = Vec[Int].new();
  var j = 0;
  while j < 10000 { a.push(n); j = j + 1; }
  var b = Vec[Float64].new();
  j = 0;
  while j < 10000 { b.push(1.0); j = j + 1; }
  var c = Vec[Int].new();
  j = 0;
  while j < 10000 { c.push(2); j = j + 1; }
  return Frame{ a: a; b: b; c: c };
}

fn consume(f: Frame) -> Int {
  return f.a.len() + f.b.len() + f.c.len();
}

fn main() -> Int {
  var i = 0;
  var sink = 0;
  while i < 200 {
    xiom_guard_heap_enter();
    let f = make_frame(i);
    sink = sink + consume(f);
    xiom_guard_heap_exit();
    i = i + 1;
  }
  io.println("done " + convert.int_to_string(sink));
  if sink != 6000000 { return 1; }
  return 0;
}
