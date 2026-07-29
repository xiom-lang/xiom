// M32: Multiple comparison operators: UInt16
fn main() -> Int {
  var a: UInt16 = 0;
  var b: UInt16 = 32768;
  var c: UInt16 = 65535;
  if a <= a && b > a && c >= b && c == 65535 as UInt16 { return 0; }
  return 1;
}
