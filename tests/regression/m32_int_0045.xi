// M32: UInt16 arithmetic
fn main() -> Int {
  var a: UInt16 = 30000;
  var b: UInt16 = 10000;
  var sum: UInt16 = a + b;
  var sub: UInt16 = a - b;
  var mul: UInt16 = a * 2;
  var div: UInt16 = a / 2;
  if sum == 40000 as UInt16 && sub == 20000 as UInt16 && mul == 60000 as UInt16 && div == 15000 as UInt16 {
    return 0;
  }
  return 1;
}
