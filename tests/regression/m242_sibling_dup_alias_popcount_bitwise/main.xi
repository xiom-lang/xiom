// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m242 (wave-97 p_sibling_dup_fn_alias): the minimal failing pair order --
// `use xiom.bits.popcount;` BEFORE a duplicate-leaf sibling bound the
// parent module's `pub fn popcount` instead of the submodule, so the
// alias-qualified call died. Pairs used to "work" only when popcount was
// imported last (its registration won the first-wins leaf slot).
module m242_sibling_dup_alias_popcount_bitwise

use xiom.bits.popcount;
use xiom.bits.bitwise;

fn main() -> Int {
  let c = popcount.next_pow2(1);
  if c != 1 { return 1; }
  let b = bitwise.rotate_left(1, 1);
  if b != 2 { return 2; }
  return 0;
}
