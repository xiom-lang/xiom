// XIOM -- Regression Test: Vec-of-struct inline storage (5c.21)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Verifies that multi-field structs stored in Vec are correctly stored
// and retrieved using memcpy-based elem_size. Covers push, index access,
// and field access on indexed elements.

module tests.ecosystem.test_vec_of_struct

pub type Point2D = { x: Int; y: Int; }
pub type Color = { r: Int; g: Int; b: Int; a: Int; }

fn main() -> Int {
  var passed = 0;
  var total = 0;

  // Test 1: Push struct to Vec and verify field access
  total = total + 1;
  var points = Vec[Point2D].new();
  var p1 = Point2D{ x: 10, y: 20 };
  points.push(p1);
  if points.len() == 1 && points[0].x == 10 && points[0].y == 20 {
    passed = passed + 1;
  }

  // Test 2: Push multiple structs and verify different indices
  total = total + 1;
  var points2 = Vec[Point2D].new();
  var a = Point2D{ x: 1, y: 2 };
  var b = Point2D{ x: 3, y: 4 };
  var c = Point2D{ x: 5, y: 6 };
  points2.push(a);
  points2.push(b);
  points2.push(c);
  if points2.len() == 3 &&
     points2[0].x == 1 && points2[0].y == 2 &&
     points2[1].x == 3 && points2[1].y == 4 &&
     points2[2].x == 5 && points2[2].y == 6 {
    passed = passed + 1;
  }

  // Test 3: Pop struct from Vec
  total = total + 1;
  var points3 = Vec[Point2D].new();
  points3.push(Point2D{ x: 99, y: 100 });
  let popped = points3.pop();
  if popped.is_some() {
    let val = popped.unwrap();
    if val.x == 99 && val.y == 100 && points3.len() == 0 {
      passed = passed + 1;
    }
  }

  // Test 4: Struct with 3+ fields
  total = total + 1;
  var colors = Vec[Color].new();
  var red = Color{ r: 255, g: 0, b: 0, a: 255 };
  colors.push(red);
  if colors[0].r == 255 && colors[0].g == 0 && colors[0].b == 0 {
    passed = passed + 1;
  }

  if passed == total { return 0; }
  return 1;
}
