// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-D09: Hash set -- membership test with array
fn main() -> Int {
  var present0: Bool = false;
  var present1: Bool = false;
  var present2: Bool = false;
  var val0: Int = 0;
  var val1: Int = 0;
  var val2: Int = 0;
  val0 = 10;
  present0 = true;
  val1 = 42;
  present1 = true;
  val2 = 100;
  present2 = true;
  if present0 == false { return 1; }
  if val0 != 10 { return 2; }
  if present1 == false { return 3; }
  if val1 != 42 { return 4; }
  if val2 != 100 { return 5; }
  return 0;
}
