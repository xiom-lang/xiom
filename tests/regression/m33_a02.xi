// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-A02: Array indexing -- access arr[0], arr[1], arr[2]
fn main() -> Int {
  var arr = [42, 17, 99];
  var a: Int = arr[0];
  var b: Int = arr[1];
  var c: Int = arr[2];
  if a == 42 && b == 17 && c == 99 { return 0; }
  return 1;
}
