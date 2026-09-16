// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-S05: Struct in array-like context -- multiple Vec3 accessed sequentially
type Vec3 = { x: Float64; y: Float64; z: Float64; }
fn sum(v: Vec3) -> Float64 { return v.x + v.y + v.z; }
fn main() -> Int {
  var a = Vec3{ x: 1.0; y: 2.0; z: 3.0; };
  var b = Vec3{ x: 4.0; y: 5.0; z: 6.0; };
  var c = Vec3{ x: 7.0; y: 8.0; z: 9.0; };
  var total: Float64 = sum(a) + sum(b) + sum(c);
  if total == 45.0 { return 0; }
  return 1;
}
