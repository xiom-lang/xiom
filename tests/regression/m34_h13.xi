// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-H13: Cast in expression chain -- multiple casts in a single expression
fn main() -> Int {
  var a: Int8 = 10;
  var b: Int16 = 20;
  var r: Int64 = (a as Int32 + b as Int32) as Int64;
  if r == 30 as Int64 { return 0; }
  return 1;
}
