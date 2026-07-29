// M32: Char to UInt8 roundtrip preserves high value (200)
fn main() -> Int {
  var u: UInt8 = 200;
  var c: Char = u as Char;
  var v: UInt8 = c as UInt8;
  if v == 200 as UInt8 { return 0; }
  return 1;
}
