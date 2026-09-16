// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn main() -> Int {
  // Normal add -- should work
  var a: Int = 10;
  var b: Int = 20;
  var c = a + b;
  if c != 30 { return 1; }

  // Normal sub -- should work
  var d = 100 - 30;
  if d != 70 { return 2; }

  // Normal mul -- should work
  var e = 6 * 7;
  if e != 42 { return 3; }

  return 0;
}
