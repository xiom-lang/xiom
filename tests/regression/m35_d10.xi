// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-D10: Hash map -- key-value storage pattern
fn main() -> Int {
  var key0: Int = 1;
  var val0: Int = 100;
  var key1: Int = 2;
  var val1: Int = 200;
  var key2: Int = 3;
  var val2: Int = 300;
  if key0 != 1 { return 1; }
  if val0 != 100 { return 2; }
  if key1 != 2 { return 3; }
  if val1 != 200 { return 4; }
  if key2 != 3 { return 5; }
  if val2 != 300 { return 6; }
  var found: Int = -1;
  if key0 == 1 { found = val0; }
  if found != 100 { return 7; }
  return 0;
}
