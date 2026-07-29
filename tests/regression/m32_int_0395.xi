// M32: Int8 to Float64 roundtrip (positive value)
fn main() -> Int {
  var a: Int8 = 100;
  var f: Float64 = a as Float64;
  var b: Int8 = f as Int8;
  if b == 100 as Int8 { return 0; }
  return 1;
}
