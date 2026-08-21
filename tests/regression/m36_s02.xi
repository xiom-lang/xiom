// M36-S02: Lexer token patterns -- identifier/keyword recognition
fn is_identifier_start(c: Int) -> Bool {
  return (c >= 97 && c <= 122) || (c >= 65 && c <= 90) || c == 95;
}
fn is_identifier_part(c: Int) -> Bool {
  return is_identifier_start(c) || (c >= 48 && c <= 57);
}
fn is_keyword(s: Str) -> Bool {
  return s == "if" || s == "else" || s == "while" || s == "return" || s == "fn" || s == "var";
}
fn validate_identifier_len(name: Str) -> Bool {
  if name.len() == 0 { return false; }
  return true;
}
fn main() -> Int {
  if !is_keyword("if") { return 1; }
  if !is_keyword("return") { return 2; }
  if !is_keyword("fn") { return 3; }
  if is_keyword("hello") { return 4; }
  if is_keyword("") { return 5; }
  if !is_identifier_start(120) { return 6; }
  if !is_identifier_part(53) { return 7; }
  if !validate_identifier_len("myVar") { return 8; }
  if validate_identifier_len("") { return 9; }
  return 0;
}
