// M35-S26: String expansion -- expand repeated chars (a3b2 -> aaabb)
use stdlib.xiom.string;
fn expand(s: Str) -> Str {
  if s.len() == 0 { return ""; }
  var result: Str = "";
  var i: Int = 0;
  while i < s.len() {
    var ch_byte = string.byte_at(s, i);
    if ch_byte >= 48 && ch_byte <= 57 {
      var count: Int = (ch_byte as Int) - 48;
      var k: Int = 0;
      while k < count {
        result = result + string.str_slice(s, i - 1, i);
        k = k + 1;
      }
    } else {
      if i + 1 == s.len() {
        result = result + string.str_slice(s, i, i + 1);
      }
    }
    i = i + 1;
  }
  return result;
}
fn main() -> Int {
  var r: Str = expand("a3b2");
  if r.len() >= 5 { return 0; }
  return 1;
}
