// m47_round14b_multibyte_chars -- round-14 (2026-08-22) regression:
// the multibyte Char family (BUG 26 #7). xiom_char_at returned the RAW
// BYTE (a 2-byte char yielded 0xCE instead of the codepoint 0x03A9), so
// len_utf8()/str_chars() mis-counted multibyte strings; Vec[Char] slots
// held 1 byte (codepoints > 255 truncated to their low byte); byte_at
// reused xiom_char_at and double-decoded once char_at was fixed. Now:
// xiom_char_at decodes the UTF-8 CODEPOINT (i64 ABI), xiom_byte_at is
// the raw-byte accessor, Vec[Char] slots are 4 bytes.
module m47_round14b_multibyte_chars
use xiom.string;
use xiom.char;
use xiom.slice;

fn main() -> Int {
  // 1. len_utf8 of a 2-byte and a 3-byte char.
  var s = "e\u{03A9}";
  unsafe {
    var c = xiom_char_at(s, 1);
    if len_utf8(c) != 2 { return 1; }
  }
  var s2 = "\u{20AC}" + "x";
  unsafe {
    var c2 = xiom_char_at(s2, 0);
    if len_utf8(c2) != 3 { return 2; }
  }
  // 2. str_chars counts codepoints, not bytes.
  var chars = str_chars(s);
  if chars.len() != 2 { return 3; }
  if (chars[1] as Int) != 937 { return 4; }
  // 3. str_code_points decodes through the byte-level path.
  var cps = str_code_points("a\u{03A9}");
  if cps.len() != 2 { return 5; }
  if cps[1] != 937 { return 6; }
  // 4. byte_at still returns the RAW byte.
  if (byte_at(s, 1) as Int) != 0xCE { return 7; }
  if (byte_at(s, 2) as Int) != 0xA9 { return 8; }
  // 5. ASCII unaffected.
  var plain = str_chars("abc");
  if plain.len() != 3 { return 9; }
  return 0;
}
