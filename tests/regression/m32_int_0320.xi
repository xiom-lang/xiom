// M32: Char value 128 cast to Int should be 128, not -128 (zext vs sext)
fn main() -> Int {
  var c: Char = '\x80';
  var n: Int = c as Int;
  if n == 128 { return 0; }
  return 1;
}
