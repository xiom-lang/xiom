// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-S12: Multiple struct types interacting
type Vec3 = { x: Int; y: Int; z: Int; }
type Transform = { scale: Int; offset: Vec3; }
fn apply(t: Transform, v: Vec3) -> Vec3 {
  return Vec3{ x: v.x * t.scale + t.offset.x; y: v.y * t.scale + t.offset.y; z: v.z * t.scale + t.offset.z; };
}
fn main() -> Int {
  var v = Vec3{ x: 1; y: 2; z: 3; };
  var off = Vec3{ x: 10; y: 20; z: 30; };
  var t = Transform{ scale: 2; offset: off; };
  var result = apply(t, v);
  if result.x == 12 && result.y == 24 && result.z == 36 { return 0; }
  return 1;
}
