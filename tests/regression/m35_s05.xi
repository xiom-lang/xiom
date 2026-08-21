// M35-S05: Remove character -- build new string without target char
use stdlib.xiom.string;
fn remove_char(s: Str, ch: Char) -> Str {
  var result: Str = "";
  var target = ch as Int;
  var i: Int = 0;
  while i < s.len() {
    if (string.byte_at(s, i) as Int) != target {
      result = result + string.str_slice(s, i, i + 1);
    }
    i = i + 1;
  }
  return result;
}
fn main() -> Int {
  if remove_char("hello", 'l') == "heo" && remove_char("abc", 'x') == "abc" && remove_char("aaa", 'a') == "" { return 0; }
  return 1;
}
