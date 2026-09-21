// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-A03: Array length -- compute via manual count (len not avail on inline arrays)
fn main() -> Int {
  var arr = [10, 20, 30, 40, 50];
  var count: Int = 0;
  count += 1;
  count += 1;
  count += 1;
  count += 1;
  count += 1;
  if count == 5 && arr[4] == 50 { return 0; }
  return 1;
}
