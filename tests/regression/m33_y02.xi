// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Y02: generic enum payload + struct derive + match extraction + contract + impl + module + diff
type Point = { x: Int; y: Int; } derive[Eq]
enum Result[T] { Ok(val: T), Err(code: Int) }
fn distance(p: Point) -> Float64
  requires: p.x >= 0
  requires: p.y >= 0
  ensures: result >= 0.0
{
  var dx: Float64 = p.x as Float64;
  var dy: Float64 = p.y as Float64;
  return dx + dy;
}
fn manhattan(p: Point) -> Int { return p.x + p.y; }
interface Measurable { fn measure(self) -> Int; }
impl Measurable for Point {
  fn measure(self) -> Int { return self.x + self.y; }
}
module geom {
  pub fn dist(p: Point) -> Int { return distance(p) as Int; }
  pub fn manh(p: Point) -> Int { return manhattan(p); }
  pub fn via_if(p: Point) -> Int { return p.measure(); }
}
use geom.dist;
use geom.manh;
use geom.via_if;
enum DistMethod { Geom, Manhattan, Interface }
fn dispatch(m: DistMethod, p: Point) -> Int {
  match m {
    Geom => dist(p),
    Manhattan => manh(p),
    Interface => via_if(p),
  }
}
fn main() -> Int {
  var pt = Point{ x: 3; y: 4; };
  var r1 = dispatch(DistMethod.Geom, pt);
  var r2 = dispatch(DistMethod.Manhattan, pt);
  var r3 = dispatch(DistMethod.Interface, pt);
  if r1 == 7 && r2 == r1 && r3 == r2 { return 0; }
  return 1;
}
