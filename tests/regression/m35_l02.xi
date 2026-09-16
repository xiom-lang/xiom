// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L02: Struct with Float64 fields -- verify float field layout
type Vec3 = { x: Float64; y: Float64; z: Float64; }

fn main() -> Int {
  var v = Vec3{ x: 1.5; y: 2.5; z: 3.5; };
  var sum: Float64 = v.x + v.y + v.z;
  if sum != 7.5 { return 1; }
  var product: Float64 = v.x * v.y * v.z;
  if product != 13.125 { return 2; }
  return 0;
}
