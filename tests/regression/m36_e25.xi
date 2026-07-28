// M36-E25: Multiple modules in one file — use multiple imported modules
use stdlib.xiom.string;
fn main() -> Int {
  var s = "hello world";
  var len = string.str_len(s);
  if len != 11 { return 1; }
  return 0;
}
