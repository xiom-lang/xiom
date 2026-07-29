// M32: UInt16 subtraction inside range (50000 - 20000 = 30000)
fn main() -> Int {
  var a: UInt16 = 50000;
  var b: UInt16 = 20000;
  var c: UInt16 = a - b;
  if c == 30000 as UInt16 { return 0; }
  return 1;
}
