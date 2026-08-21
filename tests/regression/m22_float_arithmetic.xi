// M22: Float64 arithmetic -- verify runtime results
fn main() -> Int {
  var a: Float64 = 3.0;
  var b: Float64 = 4.0;
  var sum: Float64 = a + b;
  var diff: Float64 = b - a;
  var prod: Float64 = a * b;
  var quot: Float64 = 8.0 / 2.0;
  if sum == 7.0 && diff == 1.0 && prod == 12.0 && quot == 4.0 {
    return 0;
  }
  return 1;
}
