// M32: UInt16 div at max (65535 / 1 = 65535)
fn main() -> Int {
  var a: UInt16 = 65535;
  var b: UInt16 = 1;
  var c: UInt16 = a / b;
  if c == 65535 as UInt16 { return 0; }
  return 1;
}
