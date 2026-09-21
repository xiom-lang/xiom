// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C03: if elif else (3+ branches) -- three-way branching via nested else-if
fn main() -> Int {
  var val: Int = 15;
  var result: Int = 0;
  if val < 10 { result = 1; }
  else { if val < 20 { result = 2; } else { result = 3; } }
  if result != 2 { return 1; }
  val = 5;
  if val < 10 { result = 10; }
  else { if val < 20 { result = 20; } else { result = 30; } }
  if result != 10 { return 2; }
  val = 25;
  if val < 10 { result = 100; }
  else { if val < 20 { result = 200; } else { result = 300; } }
  if result != 300 { return 3; }
  return 0;
}
