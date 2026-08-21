// M36-X27: Float precision -- floating point operations and epsilon comparisons
const EPSILON: Float64 = 0.0001;
fn approx_eq(a: Float64, b: Float64) -> Bool {
  var diff: Float64 = a - b;
  if diff < 0.0 { diff = 0.0 - diff; }
  return diff < EPSILON;
}
fn compute_pi_approx() -> Float64 {
  var pi: Float64 = 3.0;
  pi = pi + 0.14159;
  return pi;
}
fn float_ops(a: Float64, b: Float64) -> Float64 {
  return a * b + a / b - (a + b) * 0.5;
}
fn main() -> Int {
  var x: Float64 = 1.0;
  var y: Float64 = 2.0;
  var z: Float64 = x + y;
  if z != 3.0 { return 1; }
  var w: Float64 = x * y;
  if w != 2.0 { return 2; }
  var d: Float64 = y / x;
  if d != 2.0 { return 3; }
  var s: Float64 = y - x;
  if s != 1.0 { return 4; }
  var neg: Float64 = 0.0 - x;
  if neg != -1.0 { return 5; }
  var pi = compute_pi_approx();
  var pi_ref: Float64 = 3.14159;
  var diff: Float64 = pi - pi_ref;
  if diff < 0.0 { diff = 0.0 - diff; }
  if diff >= 0.001 { return 6; }
  if !approx_eq(1.0, 1.0) { return 7; }
  if approx_eq(1.0, 2.0) { return 8; }
  var r = float_ops(4.0, 2.0);
  if r != 7.0 { return 9; }
  return 0;
}
