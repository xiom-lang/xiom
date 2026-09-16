// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N2-19: 20-deep multiplication expression -- pushes expression nesting limit
fn main() -> Int {
  var r = 1 * 2 * 3 * 4 * 5 * 6 * 7 * 8 * 9 * 10 *
          11 * 12 * 13 * 14 * 15 * 16 * 17 * 18 * 19 * 20;
  if r == 2432902008176640000 { return 0; }
  return 1;
}
