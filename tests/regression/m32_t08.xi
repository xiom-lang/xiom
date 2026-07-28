// M32-T08: String with spaces — spaces preserved
fn main() -> Int {
  var s: Str = "a b c";
  var t: Str = "x   y";
  if s.len() == 5 && t.len() == 5 { return 0; }
  return 1;
}
