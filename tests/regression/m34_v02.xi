// M34-V02: Very small Float64 values (denormals / tiny)
fn main() -> Int {
  var a: Float64 = 0.000001;
  var b: Float64 = 0.0;
  if a > b && a < 1.0 {
    return 0;
  }
  return 1;
}
