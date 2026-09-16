// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_loop_capture
// BUG 22 #6 regression: an unsafe block whose INNER loop binds a name that
// SHADOWS a same-named let from an ENCLOSING loop (the stdlib str_reverse
// shape). The capture collector used to grab the OUTER loop's alloca (its
// `let c` lived inside the loop body), producing "Instruction does not
// dominate all uses!" when the unsafe-block ctx stored its address. Names
// bound inside the unsafe block are now excluded from captures.

fn main() -> Int {
  var acc = 0;
  var i = 0;
  while i < 2 {
    let c = i * 10;        // outer loop's c -- alloca used to live in the loop
    acc = acc + c;
    i = i + 1;
  }
  var out: Int = 0;
  unsafe {
    var j = 0;
    while j < 3 {
      let c = j * 2;       // the block's OWN c -- shadows the outer one
      out = out + c;
      j = j + 1;
    }
  }
  if out != 6 { return 1; }    // 0 + 2 + 4
  if acc != 10 { return 2; }   // 0 + 10
  return 0;
}
