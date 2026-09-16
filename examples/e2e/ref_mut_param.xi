// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// E2E regression: `&mut Scalar` parameters are real LLVM pointers.
// `inc(p: &mut Int)` derefs (`*p`) to read and stores through (`*p = ...`) to
// write, so the caller's `a` is mutated in place. Returns 0 on success.
module e2e_ref_mut_param

fn inc(p: &mut Int) {
  *p = *p + 1;
}

fn main() -> Int {
  var a = 1;
  inc(&mut a);
  if a == 2 {
    return 0;
  }
  return 1;
}
