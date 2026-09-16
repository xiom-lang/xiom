// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M19-R02: Enum variants with same-named fields but different types
// Verifies that Bool(val), Number(val), String(val) all extract correctly
// despite sharing the field name "val"
enum JsonValue {
  Null,
  Bool(val: Bool),
  Number(val: Float64),
  String(val: Str),
}

fn main() -> Int {
  // Test Bool variant
  var b = JsonValue.Bool(true);
  match b {
    Bool(val) => { if val != true { return 1; } }
    _ => { return 2; }
  }
  
  // Test Float64 variant
  var n = JsonValue.Number(3.14);
  match n {
    Number(val) => {
      var diff = val - 3.14;
      if diff < 0.0 { diff = -diff; }
      if diff > 0.01 { return 3; }
    }
    _ => { return 4; }
  }
  
  // Test Str variant
  var s = JsonValue.String("M19-OK");
  match s {
    String(val) => {
      if val != "M19-OK" { return 5; }
    }
    _ => { return 6; }
  }
  
  return 0;
}
