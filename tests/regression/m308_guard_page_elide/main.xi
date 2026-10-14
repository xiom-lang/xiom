// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m308 lock (REL-20261010-1554-benchmark, t3 trampoline floor): the
// guard-PAGE arm/disarm pair is a per-entry VirtualProtect (Windows) /
// mprotect (POSIX) syscall and the page is never touched by generated code.
// A confined block fn that never allocates from the guard arena must drop
// the pair (SEH/sigsetpc trampoline + guard-heap depth stay). This fixture
// exercises both shapes end to end:
//   - t3_shape: arena-free confined block re-entered 20k times (indirect
//     call per entry, the t3-hot-reload shape);
//   - early_return_block: arena-free confined block ending in `return`
//     (the marked stmt.rs Return-path disarm must also be elided);
//   - arena_block: a Vec built INSIDE unsafe keeps the guard arena + the
//     full arm/disarm pair (Copy-Out still promotes the Str).
module m308_guard_page_elide.main

use xiom.io;

fn abs_probe(x: Int) -> Int
{
  if x < 0 { return 0 - x; }
  return x;
}

fn t3_shape(cycles: Int) -> Int
{
  var fn_ptr: fn(Int) -> Int = abs_probe;
  var checksum: Int = 0;
  var i: Int = 0;
  while i < cycles {
    var j: Int = 0;
    while j < 100 {
      let arg: Int = (i * 31 + j * 17) % 1000 - 500;
      var value: Int = 0;
      unsafe { value = fn_ptr(arg); }
      checksum = checksum + value;
      j = j + 1;
    }
    i = i + 1;
  }
  return checksum;
}

fn early_return_block() -> Int
  requires: true
{
  unsafe {
    if 1 == 1 { return 7; }
    return 9;
  }
}

fn arena_block() -> Str
  requires: true
{
  unsafe {
    var v: Vec[UInt8] = Vec[UInt8].new();
    v.push(72 as UInt8);
    v.push(73 as UInt8);
    return Str::from_utf8(v);
  }
}

fn main() -> Int
{
  let checksum = t3_shape(200);
  if checksum != 4983386 {
    io.println("bad checksum=" + checksum.to_str());
    return 1;
  }

  let r = early_return_block();
  if r != 7 {
    io.println("bad early return=" + r.to_str());
    return 2;
  }

  let s = arena_block();
  if s != "HI" {
    io.println("bad str=" + s);
    return 3;
  }

  io.println("m308 guard-page elision ok");
  return 0;
}
