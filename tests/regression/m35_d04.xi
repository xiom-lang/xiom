// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-D04: Queue -- data structure verification
fn main() -> Int {
  var data0: Int = 0;
  var data1: Int = 0;
  var data2: Int = 0;
  var data3: Int = 0;
  var head: Int = 0;
  var tail: Int = 0;
  data0 = 10;
  tail = 1;
  data1 = 20;
  tail = 2;
  data2 = 30;
  tail = 3;
  data3 = 40;
  tail = 0;
  if data0 != 10 { return 1; }
  head = 1;
  if data1 != 20 { return 2; }
  head = 2;
  if data2 != 30 { return 3; }
  head = 3;
  if data3 != 40 { return 4; }
  return 0;
}
