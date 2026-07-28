// M35-T04: Char in every context — var, param, return, struct, enum, if, array, generic
type CharBox = { val: Char; }
enum CharResult { Found(c: Char), NotFound }
fn char_eq(a: Char, b: Char) -> Bool { return a == b; }
fn char_pass(c: Char) -> Char { return c; }
fn char_if(c: Char) -> Int { if c == 'A' { return 1; } return 0; }
fn char_match(c: Char) -> Int { if c == 'X' { return 10; } if c == 'Y' { return 20; } return 0; }
fn generic_char[T](x: T) -> T { return x; }
fn main() -> Int {
  var c1: Char = 'A'; var c2: Char = 'Z';
  if !char_eq(c1, 'A') { return 1; }
  if char_eq(c1, c2) { return 2; }
  if char_pass('X') != 'X' { return 3; }
  if char_if('A') != 1 { return 4; }
  if char_if('B') != 0 { return 5; }
  if char_match('X') != 10 { return 6; }
  if char_match('Y') != 20 { return 7; }
  if char_match('Q') != 0 { return 8; }
  var box: CharBox = CharBox{ val: 'K' }; if box.val != 'K' { return 9; }
  var e: CharResult = CharResult.Found('P'); match e { CharResult.Found(c) => if c != 'P' { return 10; } CharResult.NotFound => return 11; }
  var arr: Vec[Char] = ['a', 'b', 'c']; if arr[0] != 'a' { return 12; }
  var g: Char = generic_char('Z'); if g != 'Z' { return 13; }
  return 0;
}

