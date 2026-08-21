// M35-S25: String compression -- deduplicate consecutive chars via loop
use stdlib.xiom.string;
fn compress(s: Str) -> Str {
  if s.len() == 0 { return ""; }
  var result: Str = "";
  var last: UInt8 = 0;
  var i: Int = 0;
  while i < s.len() {
    var b = string.byte_at(s, i);
    if b != last {
      result = result + string.str_slice(s, i, i + 1);
      last = b;
    }
    i = i + 1;
  }
  return result;
}
fn main() -> Int {
  if compress("aabbb") == "ab" && compress("hello") == "helo" && compress("") == "" && compress("aaa") == "a" { return 0; }
  return 1;
}
