// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m244 (honest containment): raw pointer dereferences get a NULL guard
// under --overflow-checks. Inside a confined `unsafe` block the guard's
// trap surfaces through the runtime's hardware-fault path (deterministic,
// fault code 2 = illegal instruction) instead of performing the access;
// the program continues inside the language's unsafe-block contract.
// Valid references must keep working. Runs clean in both modes.
module m244_null_deref_guard

fn main() -> Int {
  // Read through a NULL pointer: guard traps inside the confined block.
  var p: *Int;
  unsafe { p = 0 as *Int; }
  var x = 0;
  unsafe { x = *p; }
  if x != 0 { return 1; }

  // Write through a NULL pointer: same containment.
  unsafe { *p = 5; }

  // Control: a real reference must NOT trip the guard.
  var ok = 41;
  var r = &ok;
  var y = *r;
  if y != 41 { return 2; }

  return 0;
}
