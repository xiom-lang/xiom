// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Function returning struct with narrow int fields
type Vec3i8 = { x: Int8; y: Int8; z: Int8; }
fn make_vec3i8() -> Vec3i8 { return Vec3i8{ x: 100 as Int8; y: -50 as Int8; z: 25 as Int8; }; }
fn main() -> Int {
  var v: Vec3i8 = make_vec3i8();
  var sum: Int = v.x as Int + v.y as Int + v.z as Int;
  if sum == 75 { return 0; }
  return 1;
}
