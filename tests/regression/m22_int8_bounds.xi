// M22: Int8 boundary values -- verify min/max compile and compare correctly
fn main() -> Int {
  var min: Int8 = -128;
  var max: Int8 = 127;
  var zero: Int8 = 0;
  // Verify min is negative and max > zero using valid operations
  if min < zero && max > zero {
    return 0;
  }
  return 1;
}
