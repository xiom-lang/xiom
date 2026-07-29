// M32: UInt8 bitwise AND/OR
fn main() -> Int {
  var a: UInt8 = 240;
  var b: UInt8 = 15;
  var and_result: UInt8 = a & b;
  var or_result: UInt8 = a | b;
  if and_result == 0 as UInt8 {
    if or_result == 255 as UInt8 { return 0; }
  }
  return 1;
}
