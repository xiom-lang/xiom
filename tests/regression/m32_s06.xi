// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-S06: Struct with derive[Eq] -- equality comparison
type Vec3 = { x: Float64; y: Float64; z: Float64; } derive[Eq]
fn main() -> Int {
  var a = Vec3{ x: 1.0; y: 2.0; z: 3.0; };
  var b = Vec3{ x: 1.0; y: 2.0; z: 3.0; };
  var c = Vec3{ x: 4.0; y: 5.0; z: 6.0; };
  if a == b && a != c { return 0; }
  return 1;
}
