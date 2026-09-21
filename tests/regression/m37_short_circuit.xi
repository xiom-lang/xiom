// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_short_circuit
// BUG 22 #1 regression: && and || must SHORT-CIRCUIT.
// The old bitwise lowering evaluated both operands -- a div-by-zero in the
// RHS trapped (0xC000001D) even when the LHS already decided the result.

fn side() -> Int {
  var z = 1 / 0; // traps if executed
  return z;
}

fn main() -> Int {
  // RHS must never run when the LHS is false
  if false && side() == 0 { return 1; }
  // RHS must never run when the LHS is true (||)
  if true || side() == 0 { } else { return 2; }
  // Result semantics
  var a = true && false;
  if a { return 3; }
  var b = false || true;
  if !b { return 4; }
  var c = 1 == 1 && 2 == 2;
  if !c { return 5; }
  var d = 1 == 2 || 3 == 3;
  if !d { return 6; }
  return 0;
}
