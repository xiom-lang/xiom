// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-H16: Cast in enum payload -- cast value before wrapping in variant
enum Value { Small(v: Int8), Big(v: Int64) }
fn main() -> Int {
  var raw: Int = 42;
  var v = Value.Small(raw as Int8);
  match v {
    Value.Small(x) => if x == 42 as Int8 { return 0; },
    _ => return 1,
  }
  return 1;
}
