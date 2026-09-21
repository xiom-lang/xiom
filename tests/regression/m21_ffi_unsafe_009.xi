// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_ffi_unsafe_009
type Node = { val: Int; next: *Node; }

  pub fn run() -> Int
  requires: true
{
    unsafe {
      var n: Node = { val: 1; next: 0 as *Node; };
      if n.val == 1 { return 0; }
      return 1;
    }
  }
use m21_ffi_unsafe_009.run;
fn main() -> Int { return run(); }
