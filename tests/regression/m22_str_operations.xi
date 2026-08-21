// M22: String operations -- length, concat, char access
fn main() -> Int {
  var s: Str = "hello";
  var t: Str = " world";
  var u: Str = s + t;
  var c: Char = 'A';
  if u.len() as Int == 11 && c as Int == 65 {
    return 0;
  }
  return 1;
}
