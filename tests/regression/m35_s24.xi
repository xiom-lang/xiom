// M35-S24: Common prefix — find longest shared prefix between two strings
use stdlib.xiom.string;
fn common_prefix_len(a: Str, b: Str) -> Int {
  var i: Int = 0;
  while i < a.len() && i < b.len() {
    if string.byte_at(a, i) != string.byte_at(b, i) { return i; }
    i = i + 1;
  }
  return i;
}
fn main() -> Int {
  if common_prefix_len("hello world", "hello there") == 6 && common_prefix_len("abc", "abd") == 2 && common_prefix_len("abc", "xyz") == 0 && common_prefix_len("", "") == 0 { return 0; }
  return 1;
}
