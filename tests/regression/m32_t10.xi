// M32-T10: Multi-char string -- longer string with concat and len
fn main() -> Int {
  var s: Str = "abcdefghij";
  var t: Str = "klmnopqrst";
  var u: Str = s + t;
  if s.len() == 10 && t.len() == 10 && u.len() == 20 { return 0; }
  return 1;
}
