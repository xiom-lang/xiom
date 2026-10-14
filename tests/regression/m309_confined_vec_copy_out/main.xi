// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m309 (REL-20261011-2320-bindings part 2): a Vec built INSIDE a confined
// (trampolined) unsafe block escapes through the block tail (or a return
// inside the block) with its data buffer on the guard arena. The block exit
// (xiom_guard_heap_exit) releases the arena wholesale, so the returned Vec
// used to dangle -- the caller's first element read faulted
// (STATUS_ACCESS_VIOLATION 0xC0000005; the bindings trampolined-wrapper
// crash class, layout/retry-dependent heap-corruption variant included).
// Fix: confined-block Copy-Out promotes the payload to the main heap
// (plan S2.11 v1) before the arena resets. Both lowering paths (return
// inside the block, block tail) plus the non-confined control are gated;
// pre-fix the two escaped reads fault, post-fix all three arms are green.
module m309_confined_vec_copy_out

use xiom.io;

// Return-inside-block path: `return buf;` in the whole-body unsafe block.
fn escape_return() -> Vec[UInt8]
  requires: true
{
  unsafe {
    var buf: Vec[UInt8] = Vec[UInt8].new();
    var i: Int = 0;
    while i < 16 {
      buf.push((i + 1) as UInt8);
      i = i + 1;
    }
    return buf;
  }
}

// Block-tail path: the unsafe block IS the fn's tail expression.
fn escape_tail() -> Vec[UInt8]
  requires: true
{
  unsafe {
    var buf: Vec[UInt8] = Vec[UInt8].new();
    var i: Int = 0;
    while i < 16 {
      buf.push((i + 129) as UInt8);
      i = i + 1;
    }
    buf
  }
}

// Control: the Vec lives on the main heap (created outside the block) and
// the return is outside any confined block -- no Copy-Out needed, must stay
// green on every baseline.
fn outside_tail() -> Vec[UInt8] {
  var buf: Vec[UInt8] = Vec[UInt8].new();
  unsafe {
    var i: Int = 0;
    while i < 16 {
      buf.push((i + 65) as UInt8);
      i = i + 1;
    }
  }
  return buf;
}

fn sum_of(v: &Vec[UInt8]) -> Int {
  var s: Int = 0;
  var i: Int = 0;
  while i < v.len() {
    s = s + v[i] as Int;
    i = i + 1;
  }
  return s;
}

fn main() -> Int {
  let a = escape_return();
  if a.len() != 16 { return 1; }
  if sum_of(&a) != 136 { return 2; }          // 1..16
  let b = escape_tail();
  if b.len() != 16 { return 3; }
  if sum_of(&b) != 2184 { return 4; }         // 129..144
  let c = outside_tail();
  if c.len() != 16 { return 5; }
  if sum_of(&c) != 1160 { return 6; }         // 65..80
  io.println("m309 confined vec copy-out: all arms green");
  return 0;
}
