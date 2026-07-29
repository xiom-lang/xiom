// M32: UInt16 modulo (50000 % 3 = 2)
fn main() -> Int {
  var a: UInt16 = 50000;
  var b: UInt16 = 3;
  var c: UInt16 = a % b;
  if c == 2 as UInt16 { return 0; }
  return 1;
}
