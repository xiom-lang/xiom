// M35-S29: Wildcard match -- match string against pattern with ? and *
use stdlib.xiom.string;
fn is_star(b: UInt8) -> Bool { return b == 42; }
fn is_quest(b: UInt8) -> Bool { return b == 63; }
fn wildcard_match(s: Str, pat: Str) -> Bool {
  var si: Int = 0;
  var pi: Int = 0;
  var star_idx: Int = -1;
  var match_idx: Int = 0;
  while si < s.len() {
    if pi < pat.len() && (is_quest(string.byte_at(pat, pi)) || string.byte_at(s, si) == string.byte_at(pat, pi)) {
      si = si + 1;
      pi = pi + 1;
    } else {
      if pi < pat.len() && is_star(string.byte_at(pat, pi)) {
        star_idx = pi;
        match_idx = si;
        pi = pi + 1;
      } else {
        if star_idx != -1 {
          pi = star_idx + 1;
          match_idx = match_idx + 1;
          si = match_idx;
        } else {
          return false;
        }
      }
    }
  }
  while pi < pat.len() && is_star(string.byte_at(pat, pi)) { pi = pi + 1; }
  return pi == pat.len();
}
fn main() -> Int {
  if wildcard_match("hello", "h*o") && wildcard_match("hello", "h?llo") && !wildcard_match("hello", "h?x") && wildcard_match("", "*") { return 0; }
  return 1;
}
