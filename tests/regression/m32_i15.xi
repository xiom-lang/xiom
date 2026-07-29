// M32: Complex arithmetic with all int types and elif/else chain
// Correct: a=1500, b=2560, c=256, d=16781532, check=4316
fn main() -> Int {
  var x8: Int8 = 15;
  var x16: Int16 = 256;
  var x32: Int32 = 65536;
  var x64: Int64 = 16777216;
  var a: Int64 = (x8 as Int64) * 100;
  var b: Int64 = (x16 as Int64) * 10;
  var c: Int64 = (x32 as Int64) / 256;
  var d: Int64 = x64 + a + b + c;
  var expected_d: Int64 = 16781532;
  var expected_check: Int64 = 4316;
  var threshold: Int64 = 10000000;
  var zero64: Int64 = 0;
  if d == expected_d {
    var check: Int64 = d - x64;
    if check == expected_check {
      return 0;
    } elif check > zero64 {
      return 1;
    } else {
      return 2;
    }
  } elif d == zero64 {
    return 3;
  } elif d < threshold {
    return 4;
  } else {
    return 5;
  }
  return 6;
}
