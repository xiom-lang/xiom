// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m242 (wave-97 p_sibling_dup_fn_alias): importing the three sibling
// submodules that export the same rotate_left/rotate_right leaves broke
// alias-qualified resolution -- `popcount.next_pow2(1)` failed because
// `use xiom.bits.popcount;` bound bits.xi's FUNCTION `popcount` instead of
// the submodule (the parent module exports a same-named fn). Module paths
// now win over same-named items, so every alias resolves and calls.
module m242_sibling_dup_alias

use xiom.bits.rotation;
use xiom.bits.popcount;
use xiom.bits.bitwise;

fn main() -> Int {
  // The reported failing call.
  let c = popcount.next_pow2(1);
  if c != 1 { return 1; }

  // All three duplicate-leaf siblings must stay independently callable.
  let r1 = rotation.rotate_left(1, 1);
  if r1 != 2 { return 2; }
  let r2 = rotation.rotate_right(2, 1);
  if r2 != 1 { return 3; }
  let b1 = bitwise.rotate_left(1, 1);
  if b1 != 2 { return 4; }
  let b2 = bitwise.rotate_right(2, 1);
  if b2 != 1 { return 5; }

  return 0;
}
