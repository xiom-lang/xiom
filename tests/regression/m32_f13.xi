// M32: Very large Float64 values (1e308)
fn main() -> Int {
  var big: Float64 = 1.0e308;
  var small: Float64 = 1.0;
  if big > small {
    return 0;
  }
  return 1;
}
