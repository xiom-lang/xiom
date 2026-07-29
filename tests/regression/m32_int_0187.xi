// M32: Char to UInt8 cast
fn main() -> Int {
  var c: Char = 'A';
  var n: UInt8 = c as UInt8;
  if n == 65 as UInt8 { return 0; }
  return 1;
}
