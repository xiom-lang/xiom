// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L26: Heap allocation pattern -- extern alloc/free declarations with null guard
extern "C" {
  fn heap_alloc(size: UInt64) -> *UInt8;
  fn heap_free(ptr: *UInt8);
}

fn main() -> Int {
  var np: *Int;
  unsafe { np = 0 as *Int; }
  if unsafe { np == (0 as *Int) } { return 0; }
  return 1;
}
