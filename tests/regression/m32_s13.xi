// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-S13: Struct with Float64 arithmetic -- operations on fields
type Vec2 = { x: Float64; y: Float64; }
fn add(a: Vec2, b: Vec2) -> Vec2 { return Vec2{ x: a.x + b.x; y: a.y + b.y; }; }
fn scale(v: Vec2, s: Float64) -> Vec2 { return Vec2{ x: v.x * s; y: v.y * s; }; }
fn dot(a: Vec2, b: Vec2) -> Float64 { return a.x * b.x + a.y * b.y; }
fn main() -> Int {
  var a = Vec2{ x: 1.5; y: 2.5; };
  var b = Vec2{ x: 3.5; y: 4.5; };
  var c = add(a, b);
  var d = scale(c, 2.0);
  var dp: Float64 = dot(a, b);
  if d.x == 10.0 && d.y == 14.0 && dp == 16.5 { return 0; }
  return 1;
}
