// M32: Float64 arithmetic -- add, sub, mul, div
fn main() -> Int {
  var a: Float64 = 10.0;
  var b: Float64 = 4.0;
  var sum: Float64 = a + b;
  var diff: Float64 = a - b;
  var prod: Float64 = a * b;
  var quot: Float64 = a / b;
  if sum == 14.0 && diff == 6.0 && prod == 40.0 && quot == 2.5 {
    return 0;
  }
  return 1;
}
