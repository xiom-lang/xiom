// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m235 (C-ORBIT-05): static `alloca` instructions emitted INSIDE a loop leak
// stack: LLVM executes an alloca each time control reaches it and frees the
// memory only at function return. This nested Vec[Page] shape (struct-element
// reads + push into another Vec[Page]) died with 0xC0000005 after ~86.7k
// iterations = ~16.6 MB of leaked 48-byte temps. The emitter now hoists every
// static alloca to the function's entry block.

module m235_loop_alloca_hoist

use xiom.io;

type Page = {
  id: Int;
  data: Vec[Int];
  checksum: Int;
}

fn main() -> Int {
  let n = 417;
  var all = Vec[Page].new();
  var i = 0;
  while i < n {
    var d = Vec[Int].new();
    d.push(i & 0xFF);
    all.push(Page{ id: i, data: d, checksum: 0 });
    i = i + 1;
  }
  var out2 = Vec[Page].new();
  var k3 = 0;
  while k3 < all.len() {
    var sup3 = false;
    var j3 = k3 + 1;
    while j3 < all.len() && !sup3 {
      if all[j3].id == all[k3].id { sup3 = true; }
      j3 = j3 + 1;
    }
    if !sup3 { out2.push(all[k3]); }
    k3 = k3 + 1;
  }
  io.println("D ok out=" + out2.len().to_string());
  return 0;
}
