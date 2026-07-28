// M35-S18: Word count — count spaces to determine word count
use stdlib.xiom.string;
fn word_count(s: Str) -> Int {
  if s.len() == 0 { return 0; }
  var count: Int = 0;
  var in_word: Bool = false;
  var space: UInt8 = 32;
  var i: Int = 0;
  while i < s.len() {
    var b = string.byte_at(s, i);
    if b != space {
      if !in_word { count = count + 1; in_word = true; }
    } else {
      in_word = false;
    }
    i = i + 1;
  }
  return count;
}
fn main() -> Int {
  if word_count("hello world") == 2 && word_count("  a  b  ") == 2 && word_count("") == 0 && word_count("single") == 1 { return 0; }
  return 1;
}
