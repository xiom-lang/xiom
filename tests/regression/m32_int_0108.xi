// M32: Int8 bitwise AND/OR/XOR at boundary
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: Int8 = 127 as Int8;
  var and_result: Int8 = a & b;
  var or_result: Int8 = a | b;
  var xor_result: Int8 = a ^ b;
  if and_result == 0 as Int8 {
    if or_result == -1 as Int8 {
      if xor_result == -1 as Int8 { return 0; }
    }
  }
  return 1;
}
