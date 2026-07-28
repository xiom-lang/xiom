// M36-C25: Every float pattern — add, sub, mul, div, cmp, cast, negate, abs, param, return, struct field, array element
fn float_add(a: Float64, b: Float64) -> Float64 { return a + b; }
fn float_sub(a: Float64, b: Float64) -> Float64 { return a - b; }
fn float_mul(a: Float64, b: Float64) -> Float64 { return a * b; }
fn float_div(a: Float64, b: Float64) -> Float64 { return a / b; }
fn float_neg(a: Float64) -> Float64 { return -a; }
fn float_cmp(a: Float64, b: Float64) -> Int {
  if a < b { return -1; }
  if a > b { return 1; }
  return 0;
}
fn float_cast(v: Int) -> Float64 { return v as Float64; }
fn float_abs(a: Float64) -> Float64 { if a < 0.0 { return -a; } return a; }
type FloatBox = { val: Float64; }
fn main() -> Int {
  if float_add(2.5, 3.5) < 5.99 || float_add(2.5, 3.5) > 6.01 { return 1; }
  if float_sub(10.0, 7.0) < 2.99 || float_sub(10.0, 7.0) > 3.01 { return 2; }
  if float_mul(2.0, 3.5) < 6.99 || float_mul(2.0, 3.5) > 7.01 { return 3; }
  if float_div(7.0, 2.0) < 3.49 || float_div(7.0, 2.0) > 3.51 { return 4; }
  if float_neg(5.0) > -4.99 { return 5; }
  if float_neg(-5.0) < 4.99 || float_neg(-5.0) > 5.01 { return 6; }
  if float_neg(0.0) != 0.0 { return 7; }
  if float_cmp(1.0, 2.0) != -1 { return 8; }
  if float_cmp(2.0, 1.0) != 1 { return 9; }
  if float_cmp(3.0, 3.0) != 0 { return 10; }
  var fc = float_cast(42);
  if fc < 41.99 || fc > 42.01 { return 11; }
  if float_cast(0) != 0.0 { return 12; }
  if float_abs(-3.14) < 3.13 || float_abs(-3.14) > 3.15 { return 13; }
  if float_abs(3.14) < 3.13 || float_abs(3.14) > 3.15 { return 14; }
  var fb = FloatBox{ val: 1.5; };
  if fb.val < 1.49 || fb.val > 1.51 { return 15; }
  return 0;
}
