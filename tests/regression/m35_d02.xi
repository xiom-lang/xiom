// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-D02: Stack overflow -- push beyond capacity
fn main() -> Int {
  var data0: Int = 0;
  var data1: Int = 0;
  var data2: Int = 0;
  var data3: Int = 0;
  var top: Int = -1;
  top = 0;
  data0 = 10;
  top = 1;
  data1 = 20;
  top = 2;
  data2 = 30;
  top = 3;
  data3 = 40;
  if top != 3 { return 1; }
  if data0 != 10 { return 2; }
  if data3 != 40 { return 3; }
  top = 2;
  top = 1;
  top = 0;
  top = -1;
  if top != -1 { return 4; }
  return 0;
}
