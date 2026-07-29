// M32: Operator precedence: UInt16 (a + b) * c
fn main() -> Int {
  var a: UInt16 = 40000;
  var b: UInt16 = 25535;
  var c: UInt16 = 1;
  var r: UInt16 = a + b * c;
  if r == 65535 as UInt16 { return 0; }
  return 1;
}
