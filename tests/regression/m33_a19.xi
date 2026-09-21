// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-A19: Array with generic -- generic identity on array elements
fn id[T](x: T) -> T { return x; }
fn main() -> Int {
  var arr = [42, 99, 17];
  if id(arr[0]) == 42 && id(arr[2]) == 17 { return 0; }
  return 1;
}
