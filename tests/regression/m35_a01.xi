// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-A01: Binary search on sorted array -- inline divide-and-conquer in main
fn main() -> Int {
  var arr = [1, 3, 5, 7, 9, 11, 13, 15, 17, 19];
  var n: Int = 10;
  var target: Int = 7;
  var found: Int = -1;
  var lo: Int = 0;
  var hi: Int = n - 1;
  while lo <= hi {
    var mid: Int = lo + (hi - lo) / 2;
    var val: Int = arr[mid];
    if val == target { found = mid; lo = hi + 1; } else {
      if val < target { lo = mid + 1; } else { hi = mid - 1; }
    }
  }
  if found != 3 { return 1; }
  target = 1; found = -1; lo = 0; hi = n - 1;
  while lo <= hi {
    var mid2: Int = lo + (hi - lo) / 2;
    var val2: Int = arr[mid2];
    if val2 == target { found = mid2; lo = hi + 1; } else {
      if val2 < target { lo = mid2 + 1; } else { hi = mid2 - 1; }
    }
  }
  if found != 0 { return 2; }
  target = 6; found = -1; lo = 0; hi = n - 1;
  while lo <= hi {
    var mid3: Int = lo + (hi - lo) / 2;
    var val3: Int = arr[mid3];
    if val3 == target { found = mid3; lo = hi + 1; } else {
      if val3 < target { lo = mid3 + 1; } else { hi = mid3 - 1; }
    }
  }
  if found != -1 { return 3; }
  return 0;
}
