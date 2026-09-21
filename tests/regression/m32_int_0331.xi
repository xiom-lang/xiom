// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Nested type alias with narrow int and field widening
type Pixel = { r: Int8; g: Int8; b: Int8; a: Int8; }
fn main() -> Int {
  var p: Pixel = Pixel{ r: 100 as Int8; g: -50 as Int8; b: 0 as Int8; a: 127 as Int8; };
  var gray: Int = p.r as Int + p.g as Int + p.b as Int;
  if gray == 50 { return 0; }
  return 1;
}
