// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m249 (stdlib slice-bound C001): `is_sorted[T: Ord](items: &Slice[T])`
// called with a Slice[Int] local used to infer T="Slice" (type_from_ast
// strips the Slice wrapper, so the bare-T branch read the argument's
// outer type) and C001'd "Slice does not implement Ord". T now infers
// from the element: the mono is core.is_sorted_Int.
//
// REMAINING OPEN (documented in COMPILER_BUGS m249): the call-site ABI
// still passes a %struct.Slice value where the mono'd `&Slice[T]` def
// expects its %struct.Vec lowering; clang rejects the call. This fixture
// is exercised at IR level until the Slice->Vec bridge lands.
module m249_slice_bound_infer

use xiom.core;
use xiom.array;

fn main() -> Int {
  let arr = [1, 2, 3];
  let s = array.as_slice(&arr);
  if core.is_sorted(&s) == false { return 1; }
  return 0;
}
