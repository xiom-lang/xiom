// M36-X05: Generic dispatch -- generic functions with multiple type parameters
fn max(a: Int, b: Int) -> Int {
  if a > b { return a; }
  return b;
}
fn min(a: Int, b: Int) -> Int {
  if a < b { return a; }
  return b;
}
fn clamp(val: Int, lo: Int, hi: Int) -> Int {
  var v: Int = val;
  if v < lo { v = lo; }
  if v > hi { v = hi; }
  return v;
}
fn max_f(a: Float64, b: Float64) -> Float64 {
  if a > b { return a; }
  return b;
}
fn clamp_f(val: Float64, lo: Float64, hi: Float64) -> Float64 {
  var v: Float64 = val;
  if v < lo { v = lo; }
  if v > hi { v = hi; }
  return v;
}
fn main() -> Int {
  if max(10, 20) != 20 { return 1; }
  if max_f(3.14, 2.71) != 3.14 { return 2; }
  if min(5, 3) != 3 { return 3; }
  if clamp(5, 0, 10) != 5 { return 4; }
  if clamp(-5, 0, 10) != 0 { return 5; }
  if clamp(15, 0, 10) != 10 { return 6; }
  if clamp_f(1.5, 0.0, 1.0) != 1.0 { return 7; }
  return 0;
}
