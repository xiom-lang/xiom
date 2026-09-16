// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-C11: Contract with generic -- identity with ensures equality
fn id[T](x: T) -> T
  ensures: result == x
{
  return x;
}
fn main() -> Int {
  var r: Int = id(42);
  if r == 42 { return 0; }
  return 1;
}
