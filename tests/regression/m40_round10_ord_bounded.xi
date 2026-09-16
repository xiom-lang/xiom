// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// m40_round10_ord_bounded -- round-10 (2026-08-20) regression:
// the checker's builtin Ord/Bounded interface resolution (C001 gap).
// The stdlib Ord tower (impl Ord[Int] etc. with compare+cmp) now registers;
// cmp/min/max are compiler-derivable for primitives; generic-param static
// receivers (T.max_value() in mono'd bodies) resolve via current_type_map
// to the module-qualified impl (precision.Int.max_value); checked_add/sub/
// mul/div + saturating ops work through the Bounded + Ord bounds.
module m40_round10_ord_bounded
use xiom.num;

fn main() -> Int {
  // checked_add overflow guard uses T.zero/max_value/min_value + Ord ops.
  match num.checked_add(10, 20) {
    Some(v) => { if v != 30 { return 1; } }
    None => { return 2; }
  }
  // overflow must return None (Int64::MAX + 1).
  match num.checked_add(9223372036854775807, 1) {
    Some(_) => { return 3; }
    None => { }
  }
  match num.checked_sub(50, 20) {
    Some(v) => { if v != 30 { return 4; } }
    None => { return 5; }
  }
  match num.checked_mul(5, 6) {
    Some(v) => { if v != 30 { return 6; } }
    None => { return 7; }
  }
  // saturating ops.
  if num.saturating_add(10, 20) != 30 { return 8; }
  if num.saturating_sub(50, 20) != 30 { return 9; }
  // Ord tower: compare/cmp on primitives.
  if Int.compare(1, 2) != -1 { return 10; }
  if Int.cmp(2, 2) != 0 { return 11; }
  if Int.cmp(3, 2) != 1 { return 12; }
  return 0;
}
