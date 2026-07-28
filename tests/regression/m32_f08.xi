// M32: Float32 to Int conversion (truncation)
fn main() -> Int {
  var f: Float32 = 99.9;
  var i: Int = f as Int;
  if i == 99 {
    return 0;
  }
  return 1;
}
