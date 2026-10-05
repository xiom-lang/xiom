// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m192: a confined `unsafe` block re-entered from a loop must reuse one ctx
// alloca hoisted to the fn entry. Emitted inline it leaked 32 bytes of stack
// per execution; 262,144 entries exhaust the 8 MB reserve (0xC0000005).
// This runs 500,000 entries (>16 MB of pre-fix stack) and checks the sum.
use xiom.io;

fn main() {
  var sum: Int = 0;
  var i: Int = 0;
  while i < 500000 {
    let step: Int = i % 7;
    unsafe {
      sum = sum + step + i % 2;
    }
    i = i + 1;
  }
  if sum == 1749994 {
    io.println("m192 ok");
  } else {
    io.println("m192 bad sum=" + sum.to_str());
    io.exit(1);
  }
}
