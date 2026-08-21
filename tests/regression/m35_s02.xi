// M35-S02: Palindrome check -- reverse and compare
use stdlib.xiom.string;
fn reverse(s: Str) -> Str {
  var result: Str = "";
  var i = s.len() - 1;
  while i >= 0 {
    result = result + string.str_slice(s, i, i + 1);
    i = i - 1;
  }
  return result;
}
fn is_palindrome(s: Str) -> Bool {
  return s == reverse(s);
}
fn main() -> Int {
  if is_palindrome("racecar") && is_palindrome("") && is_palindrome("aa") && !is_palindrome("hello") { return 0; }
  return 1;
}
