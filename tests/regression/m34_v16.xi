// M34-V16: Float parameter chain through multiple functions
fn add2(a: Float64, b: Float64) -> Float64 { return a + b; }
fn mul2(a: Float64, b: Float64) -> Float64 { return a * b; }
fn sub2(a: Float64, b: Float64) -> Float64 { return a - b; }
fn div2(a: Float64, b: Float64) -> Float64 { return a / b; }
fn combine(a: Float64, b: Float64) -> Float64 {
  var s: Float64 = add2(a, b);
  var m: Float64 = mul2(s, 2.0);
  var d: Float64 = div2(m, 3.0);
  return sub2(d, 1.0);
}
fn main() -> Int {
  var a: Float64 = 5.0;
  var b: Float64 = 7.0;
  var result: Float64 = combine(a, b);
  if result == 7.0 {
    return 0;
  }
  return 1;
}
