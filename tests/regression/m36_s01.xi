// M36-S01: Lexer char classification — is_alpha, is_digit, is_whitespace, is_operator
fn is_alpha(c: Char) -> Bool {
  return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
}
fn is_digit(c: Char) -> Bool {
  return c >= '0' && c <= '9';
}
fn is_whitespace(c: Char) -> Bool {
  return c == ' ' || c == '\n' || c == '\t' || c == '\r';
}
fn is_operator(c: Char) -> Bool {
  return c == '+' || c == '-' || c == '*' || c == '/' || c == '=' || c == '<' || c == '>' || c == '!';
}
fn main() -> Int {
  if !is_alpha('a') { return 1; }
  if !is_alpha('Z') { return 2; }
  if !is_alpha('_') { return 3; }
  if is_alpha('1') { return 4; }
  if !is_digit('0') { return 5; }
  if !is_digit('9') { return 6; }
  if is_digit('a') { return 7; }
  if !is_whitespace(' ') { return 8; }
  if !is_whitespace('\n') { return 9; }
  if is_whitespace('x') { return 10; }
  if !is_operator('+') { return 11; }
  if !is_operator('=') { return 12; }
  if is_operator('a') { return 13; }
  return 0;
}
