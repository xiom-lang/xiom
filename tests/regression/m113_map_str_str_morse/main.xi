// m113 (L8-14): Map[Str, Str] morse flow -- deterministically trapped with
// 0xC000001D. `param.get(k).unwrap_or("?")` resolved the payload type only
// for Ident receivers; with a CALL receiver the tracked `&Map[Str, Str]`
// param had lost its args ("&Map") so the Str payload escaped as a raw i64
// handle and `result + code` stringified pointers. Fixes:
//   * ref_preserving_name keeps generic args through references;
//   * generic_container_last_arg accepts reference-qualified containers;
//   * unwrap/unwrap_or resolve call receivers via scrutinee_payload_xiom and
//     unbox aggregate payloads.
module m113.main

use xiom.io;
use xiom.convert.tostring;

fn build_morse_map() -> Map[Str, Str] {
  var m: Map[Str, Str] = Map[Str, Str].new();
  m.insert("H", "....");  m.insert("I", "..");
  m.insert("O", "---");   m.insert("S", "...");
  m
}

fn build_reverse(morse: &Map[Str, Str]) -> Map[Str, Str] {
  var rev: Map[Str, Str] = Map[Str, Str].new();
  var letters: Vec[Str] = Vec[Str].new();
  letters.push("S"); letters.push("O"); letters.push("S");
  letters.push("H"); letters.push("I");
  var i = 0;
  while i < letters.len() {
    let ch = letters.get(i).unwrap();
    let code = morse.get(ch).unwrap();
    rev.insert(code, ch);
    i = i + 1;
  };
  rev
}

fn encode_word(word: Str, morse: &Map[Str, Str]) -> Str {
  var result: Str = "";
  var i = 0;
  while i < word.len() {
    let ch = word.char_at(i);
    let code = morse.get(to_string_char(ch)).unwrap_or("?");
    result = result + code;
    if i < word.len() - 1 { result = result + " "; };
    i = i + 1;
  };
  result
}

fn decode_morse(morse_str: Str, rev: &Map[Str, Str]) -> Str {
  var result: Str = "";
  var token: Str = "";
  var i = 0;
  while i < morse_str.len() {
    let ch = morse_str.char_at(i);
    if ch == ' ' {
      let letter = rev.get(token).unwrap_or("?");
      result = result + letter;
      token = "";
    } else {
      token = token + to_string_char(ch);
    };
    i = i + 1;
  };
  if token != "" {
    let letter = rev.get(token).unwrap_or("?");
    result = result + letter;
  };
  result
}

fn main() -> Int {
  let morse_map = build_morse_map();
  let encoded = encode_word("SOS", &morse_map);
  if encoded != "... --- ..." { return 1; }
  io.println(encoded.to_str());

  let rev_map = build_reverse(&morse_map);
  let decoded = decode_morse("... --- ...", &rev_map);
  if decoded != "SOS" { return 2; }
  io.println(decoded.to_str());

  let hi_encoded = encode_word("HI", &morse_map);
  if hi_encoded != ".... .." { return 3; }
  let hi_decoded = decode_morse(".... ..", &rev_map);
  if hi_decoded != "HI" { return 4; }
  io.println(hi_decoded.to_str());
  return 0;
}
