// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-C08: Every module pattern with every visibility -- pub/non-pub functions, types, constants; flat, nested, re-export
const PI: Float64 = 3.14159;
module math {
  pub const E: Float64 = 2.71828;
  pub fn double(x: Int) -> Int { return x * 2; }
  pub fn square(x: Int) -> Int { return x * x; }
  fn helper_add3(x: Int) -> Int { return x + 3; }
  pub fn triple_via_helper(x: Int) -> Int {
    var a = helper_add3(x);
    return a + x - 3;
  }
  pub type Scores = { a: Int; b: Int; } derive[Eq]
  pub fn sum_scores(s: Scores) -> Int { return s.a + s.b; }
}
module geom {
  pub fn area(w: Int, h: Int) -> Int { return w * h; }
  pub fn perimeter(w: Int, h: Int) -> Int { return 2 * (w + h); }
  pub fn is_square(w: Int, h: Int) -> Bool { return w == h; }
  module shape2d {
    pub fn circle_area(r: Float64) -> Float64 { return PI * r * r; }
    pub fn rect_area(w: Int, h: Int) -> Int { return w * h; }
  }
}
use math.double;
use math.square;
use math.triple_via_helper;
use math.sum_scores;
use math.Scores;
use math.E;
use geom.area;
use geom.perimeter;
use geom.is_square;
use geom.shape2d.circle_area;
use geom.shape2d.rect_area;
fn main() -> Int {
  if double(21) != 42 { return 1; }
  if square(9) != 81 { return 2; }
  if triple_via_helper(4) != 8 { return 3; }
  var s = Scores{ a: 10; b: 20; };
  if sum_scores(s) != 30 { return 4; }
  var circ = circle_area(1.0);
  if circ < 3.14 || circ > 3.15 { return 5; }
  if rect_area(5, 3) != 15 { return 6; }
  if area(4, 7) != 28 { return 7; }
  if perimeter(3, 5) != 16 { return 8; }
  if !is_square(5, 5) { return 9; }
  if is_square(3, 4) { return 10; }
  if E < 2.71 || E > 2.72 { return 11; }
  return 0;
}
