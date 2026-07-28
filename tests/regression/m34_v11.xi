// M34-V11: Float equality with epsilon (abs(a-b) < 0.0001)
fn abs(x: Float64) -> Float64 {
  if x < 0.0 { return -x; }
  return x;
}
fn main() -> Int {
  var a: Float64 = 0.1 + 0.2;
  var b: Float64 = 0.3;
  var diff: Float64 = a - b;
  if diff < 0.0 { diff = -diff; }
  if diff < 0.0001 {
    return 0;
  }
  return 1;
}
