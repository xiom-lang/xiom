// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_labeled_loops
// Labeled break/continue: `@label: while ...` + `break @label;` /
// `continue @label;`. The label field previously never reached the
// codegen's loop stack, so `break @label` fell back to the innermost loop.

fn main() -> Int {
  // labeled break from a NESTED loop to the outer loop
  var count = 0;
  @outer: while true {
    var j = 0;
    while j < 10 {
      j = j + 1;
      if j == 3 { break @outer; }
      count = count + 1;
    }
  }
  if count != 2 { return 1; }
  // labeled continue skips the rest of the OUTER iteration
  var hits = 0;
  @outer2: while hits < 5 {
    hits = hits + 1;
    var j = 0;
    while j < 3 {
      j = j + 1;
      if j == 2 { continue @outer2; }
    }
    hits = hits + 100; // never reached
  }
  if hits != 5 { return 2; }
  // plain break / continue unchanged
  var i = 0;
  while i < 10 { if i == 3 { break; } i = i + 1; }
  if i != 3 { return 3; }
  var s = 0;
  var k = 0;
  while k < 5 { k = k + 1; if k == 3 { continue; } s = s + 1; }
  if s != 4 { return 4; }
  return 0;
}
