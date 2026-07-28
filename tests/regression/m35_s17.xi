// M35-S17: Extract substring — build substring from start to end index
use stdlib.xiom.string;
fn substring(s: Str, start: Int, end: Int) -> Str {
  return string.str_slice(s, start, end);
}
fn main() -> Int {
  if substring("hello world", 0, 5) == "hello" && substring("hello world", 6, 11) == "world" && substring("abc", 0, 0) == "" { return 0; }
  return 1;
}
