// M35-S01: String reverse — manual loop building reversed string
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
fn main() -> Int {
  if reverse("hello") == "olleh" && reverse("") == "" && reverse("a") == "a" { return 0; }
  return 1;
}
