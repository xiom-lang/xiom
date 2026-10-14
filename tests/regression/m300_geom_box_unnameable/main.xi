// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m300 lock (REL-20261010-1549-stdlib-2): xiom.geom.geometry_3d declares
// `pub type Box = { min: Point3; max: Point3; }` but the leaf name was
// shadowed by xiom.core's private `Box[T]`, so consumer code could not name
// the geometry_3d type: `Box{...}` resolved to the core type (T001 "type
// 'Box' has no field 'min'"), and neither qualified `geometry_3d.Box{...}`
// (T001 "unknown type") nor `use ...Box as GBox;` (T001 "unknown type
// 'GBox'") worked, leaving aabb_intersection / aabb_contains /
// ray_box_intersection uncallable from any other module. The fix resolves
// both spellings (and the call-site param check compares by registered-type
// identity, not spelling). Mirrors the archived witness
// p_geom_box_unnameable.xi in the stdlib known_failures corpus.
module m300_geom_box_unnameable.main

use xiom.geom.geometry_3d;
use xiom.geom.geometry_3d.Point3;

fn p3(x: Float64, y: Float64, z: Float64) -> Point3 {
  return Point3{ x: x; y: y; z: z; };
}

fn main() -> Int {
  var b = geometry_3d.Box{ min: p3(0.0, 0.0, 0.0); max: p3(1.0, 1.0, 1.0); };
  if !geometry_3d.aabb_contains(b, p3(0.5, 0.5, 0.5)) { return 1; }
  var b2 = geometry_3d.Box{ min: p3(0.5, 0.5, 0.5); max: p3(2.0, 2.0, 2.0); };
  if !geometry_3d.aabb_intersection(b, b2) { return 2; }
  return 0;
}
