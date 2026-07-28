// M36-X10: --check mode patterns — syntax-only validation structures
enum Token { Ident, Number, String, LParen, RParen, LBrace, RBrace, Comma, Semicolon, Arrow, Eq, Plus, Minus, Star, Slash, Dot, Colon, Bang, Amp, Pipe, Lt, Gt, Eof }
fn token_repr(t: Token) -> Int {
  match t {
    Token.Ident => 1, Token.Number => 2, Token.String => 3,
    Token.LParen => 4, Token.RParen => 5, Token.LBrace => 6,
    Token.RBrace => 7, Token.Comma => 8, Token.Semicolon => 9,
    Token.Arrow => 10, Token.Eq => 11, Token.Plus => 12,
    Token.Minus => 13, Token.Star => 14, Token.Slash => 15,
    Token.Dot => 16, Token.Colon => 17, Token.Bang => 18,
    Token.Amp => 19, Token.Pipe => 20, Token.Lt => 21,
    Token.Gt => 22, Token.Eof => 0,
  }
}
fn is_binary_op(t: Token) -> Bool {
  if t == Token.Plus { return true; }
  if t == Token.Minus { return true; }
  if t == Token.Star { return true; }
  if t == Token.Slash { return true; }
  if t == Token.Eq { return true; }
  if t == Token.Lt { return true; }
  if t == Token.Gt { return true; }
  return false;
}
fn precedence(t: Token) -> Int {
  if t == Token.Star || t == Token.Slash { return 2; }
  if t == Token.Plus || t == Token.Minus { return 1; }
  return 0;
}
fn main() -> Int {
  if !is_binary_op(Token.Plus) { return 1; }
  if !is_binary_op(Token.Minus) { return 2; }
  if is_binary_op(Token.Ident) { return 3; }
  if precedence(Token.Star) != 2 { return 4; }
  if precedence(Token.Plus) != 1 { return 5; }
  if precedence(Token.Ident) != 0 { return 6; }
  if token_repr(Token.Ident) != 1 { return 7; }
  if token_repr(Token.Eof) != 0 { return 8; }
  return 0;
}
