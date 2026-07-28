// M34-V04: Negative values at boundaries (-1e100, -1e200, -1.0)
fn main() -> Int {
  var a: Float64 = -1.0e100;
  var b: Float64 = -1.0e200;
  var c: Float64 = -1.0;
  if a < 0.0 && b < 0.0 && c < 0.0 && a > b {
    return 0;
  }
  return 1;
}
