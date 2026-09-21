// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T08: Nested arrays via flat access
fn main() -> Int {
  var a: Vec[Int] = [1, 2, 3];
  if a[0] != 1 { return 1; }
  if a[1] != 2 { return 2; }
  if a[2] != 3 { return 3; }
  return 0;
}

