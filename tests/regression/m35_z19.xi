// M35-Z19: generic+recursion+enum+closure+match+contract+derive+module+while+compound_assign
type Boxed = { len: Int; src: Int; } derive[Eq]
enum Cmd { Pack, Unpack, Send }
fn encode[T](n: Int) -> Int {
  var r = n;
  var shift = 4;
  r = r + shift;
  return r;
}
fn tree_calc(n: Int) -> Int {
  if n <= 0 { return 1; }
  return n * tree_calc(n - 2);
}
fn apply_cmd(c: Cmd, v: Int) -> Int
  requires: v >= 0
  ensures: result >= 0
{
  match c { Pack => v * 2, Unpack => v / 2, Send => v }
}
module msg {
  pub fn encode_val(n: Int) -> Int { return encode(n); }
  pub fn calc(n: Int) -> Int { return tree_calc(n); }
  pub fn dispatch(c: Cmd, v: Int) -> Int { return apply_cmd(c, v); }
}
use msg.encode_val;
use msg.calc;
use msg.dispatch;
fn main() -> Int {
  var b = Boxed{ len: 5; src: 3; };
  var ev = encode_val(b.src);
  var cv = calc(b.len);
  var d1 = dispatch(Cmd.Pack, b.len);
  var d2 = dispatch(Cmd.Unpack, d1);
  var f = |x| x + b.src;
  var i = 0;
  var acc = 0;
  while i < 3 { acc = f(acc); i += 1; }
  if ev == 7 && cv == 15 && d2 == 5 && acc == 9 { return 0; }
  return 1;
}
