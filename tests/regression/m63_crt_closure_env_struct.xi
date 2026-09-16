// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M63 (CRT-layout family #1): closure env struct malloc under-allocation.
// The closure creation site allocated 8 bytes PER CAPTURE regardless of the
// captured value's LLVM type -- capturing a STRUCT (%struct.Range = 16B)
// overflowed the malloc'd tail by 8 bytes, corrupting the heap: the whole
// clang -O2/MSVC-CRT layout family (smoke_iter_collect startup AV, m34_y15/
// y20, "flips with unrelated stdlib code"). Fix: env size = 8 + sum of real
// llvm_type_byte_size per capture field.
module m63_crt_closure_env_struct

use xiom.iter;

fn main() -> Int {
  // Empty-range collect: the closure env captures the Range STRUCT by value
  // (16 bytes) -- pre-fix the 16-byte malloc overflowed on the store.
  var empty = iter.range(0, 0).collect();
  if empty.len() != 0 { return 1; }
  var items = iter.range(1, 6).collect();
  if items.len() != 5 { return 2; }
  var i: Int = 0;
  while i < 5 {
    match items.get(i) {
      Some(v) => { if v != i + 1 { return 3; } },
      None => { return 4; },
    };
    i = i + 1;
  }
  return 0;
}
