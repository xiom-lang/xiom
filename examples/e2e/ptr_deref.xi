// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// E2E regression: raw pointer deref round-trip via `ptr.from_mut` / `*p`.
// Take the address of a scalar local, write through it, then observe the
// mutation through the original binding. Locks in `*T` as a real LLVM pointer.
// Exit 0 on success.
module e2e_ptr_deref

use xiom.ptr;

fn bump(p: &mut Int) {
  *p = *p + 2;
}

fn main() -> Int {
  var x = 40;
  // `&mut x` reaches `bump` as a real pointer; the write is visible in `x`.
  bump(&mut x);
  if x != 42 {
    return 1;
  }
  return 0;
}
