// M32: Char to Int cast
fn main() -> Int {
  var c: Char = '\x7f';
  var n: Int8 = c as Int8;
  if n == 127 as Int8 { return 0; }
  return 1;
}
