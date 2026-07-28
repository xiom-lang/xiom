// M35-S21: Shortest word — find word with minimum non-zero length
use stdlib.xiom.string;
fn shortest_word_len(s: Str) -> Int {
  if s.len() == 0 { return 0; }
  var min_len: Int = s.len();
  var curr_len: Int = 0;
  var space: UInt8 = 32;
  var i: Int = 0;
  while i < s.len() {
    if string.byte_at(s, i) != space {
      curr_len = curr_len + 1;
    } else {
      if curr_len > 0 && curr_len < min_len { min_len = curr_len; }
      curr_len = 0;
    }
    i = i + 1;
  }
  if curr_len > 0 && curr_len < min_len { min_len = curr_len; }
  if min_len == s.len() { return 0; }
  return min_len;
}
fn main() -> Int {
  if shortest_word_len("the quick brown fox") == 3 && shortest_word_len("a bb ccc") == 1 && shortest_word_len("") == 0 { return 0; }
  return 1;
}
