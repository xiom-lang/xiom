// fuzz seed: arithmetic + strings + comments (lexer surface).
fn main() -> Int {
  let x = 0x2A + 0b1010 + 1_000;
  let s = "esc \" \\ \n \t" + 'q';
  // line comment
  /* block /* nested */ comment */
  return x;
}
