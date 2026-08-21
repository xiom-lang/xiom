// M34-N11: Enum with Clone+Eq -- clone then compare equality
enum Token { Plus, Minus, Star, Slash } derive[Eq, Clone]
fn main() -> Int {
  var a = Token.Plus;
  var b = a.clone();
  var c = Token.Slash;
  var d = c.clone();
  if a == b && c == d && a != c { return 0; }
  return 1;
}
