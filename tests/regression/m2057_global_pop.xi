// m2057 (REL-20261019-2057-engine): module-global Vec.pop mutated a COPY.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `var g_q: Vec[Int]` + `g_q.pop()` compiled the global as a value, copied it
// into a scratch alloca, popped the copy and never stored back -- `g_q.len()`
// stayed 1. The receiver is now resolved to its authoritative header pointer
// (global/index/handle mutate in place; locals still round-trip). This locks
// the global plus the local, struct-field, and `&mut Vec` receivers.

module m2057_global_pop

use xiom.io;

type Queue = { items: Vec[Int]; }

var g_q: Vec[Int] = Vec[Int].new();

fn drain(v: &mut Vec[Int]) -> Int {
  let p = v.pop();
  match p {
    Some(_) => { return v.len(); }
    None => { return -1; }
  }
}

fn main() -> Int {
  g_q.push(1);
  let p = g_q.pop();
  if g_q.len() != 0 { return 1; }
  match p {
    Some(x) => { if x != 1 { return 2; } }
    None => { return 3; }
  }

  var v = Vec[Int].new();
  v.push(7);
  let q = v.pop();
  if v.len() != 0 { return 4; }
  match q {
    Some(x) => { if x != 7 { return 5; } }
    None => { return 6; }
  }

  var box = Queue{ items: Vec[Int].new() };
  box.items.push(8);
  let r = box.items.pop();
  if box.items.len() != 0 { return 7; }
  match r {
    Some(x) => { if x != 8 { return 8; } }
    None => { return 9; }
  }

  var w = Vec[Int].new();
  w.push(9);
  let n = drain(&mut w);
  if n != 0 { return 10; }
  if w.len() != 0 { return 11; }
  io.println("pop_ok");
  return 0;
}
