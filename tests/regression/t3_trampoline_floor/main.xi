// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// t3-hot-reload trampoline-floor lock (REL-20261010-1554-benchmark).
// 1,000,000 confined-block entries, one indirect call each: the exact shape
// that used to pay ~2 us per entry (sigsetjmp savemask + trap round-trip).
// The runtime must keep the POSIX hot path syscall-free; the e2e test also
// asserts a generous wall budget so a 2 us/entry regression cannot return.
module t3_trampoline_floor.main

use xiom.io;

fn ident(x: Int) -> Int
{
  return x;
}

fn main() -> Int
{
  var fn_ptr: fn(Int) -> Int = ident;
  var checksum: Int = 0;
  var i: Int = 0;
  while i < 1000000 {
    var value: Int = 0;
    unsafe { value = fn_ptr(i); }
    checksum = checksum + value;
    i = i + 1;
  }
  // sum(0..999999) = 499999500000
  if checksum != 499999500000 {
    io.println("bad checksum=" + checksum.to_str());
    return 1;
  }
  return 0;
}
