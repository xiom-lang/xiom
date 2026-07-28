// M35-S13: String prefix check — verify s starts with prefix character-by-character
use stdlib.xiom.string;
fn starts_with(s: Str, prefix: Str) -> Bool {
  if prefix.len() > s.len() { return false; }
  if prefix.len() == 0 { return true; }
  var i: Int = 0;
  while i < prefix.len() {
    if string.byte_at(s, i) != string.byte_at(prefix, i) { return false; }
    i = i + 1;
  }
  return true;
}
fn main() -> Int {
  if starts_with("hello world", "hello") && starts_with("abc", "") && !starts_with("abc", "abd") && starts_with("abc", "abc") { return 0; }
  return 1;
}
