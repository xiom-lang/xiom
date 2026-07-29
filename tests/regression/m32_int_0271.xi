// M32: UInt8 subtraction in unsigned range (200 - 50 = 150)
fn main() -> Int {
  var a: UInt8 = 200;
  var b: UInt8 = 50;
  var c: UInt8 = a - b;
  if c == 150 as UInt8 { return 0; }
  return 1;
}
