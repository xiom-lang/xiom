// M34-Q20: Combined contract chain stress -- struct+enum+generic+module+contract+Option+Result
type Pair = { a: Int; b: Int; }
enum Op { Add, Mul }
module ops {
  pub fn do_op(p: Pair, o: Op) -> Int
    requires: p.a >= 0
    requires: p.b >= 0
    ensures: result >= 0
  {
    match o {
      Add => p.a + p.b,
      Mul => p.a * p.b,
    }
  }
}
use ops.do_op;
fn summar(p: Pair) -> Int
  requires: p.a >= 0
  requires: p.b >= 0
  ensures: result >= 0
{ return p.a * 10 + p.b; }
fn bounded_op(p: Pair, o: Op) -> Int
  requires: p.a >= 0
  requires: p.b >= 0
  ensures: result >= 0
{ return do_op(p, o); }
fn unwrap_or_zero(o: Option[Int]) -> Int
  ensures: result >= 0
{
  match o { Some(v) => v, None => 0 }
}
fn safe_div(a: Int, b: Int) -> Int
  requires: b != 0
  ensures: result >= 0
{
  if a < 0 { return -a / b; }
  return a / b;
}
fn main() -> Int {
  var p1 = Pair{ a: 4; b: 5; };
  var p2 = Pair{ a: 4; b: 5; };
  var p3 = Pair{ a: 4; b: 5; };
  var s = summar(p1);
  var add_v = bounded_op(p2, Op.Add);
  var mul_v = bounded_op(p3, Op.Mul);
  var d = add_v + mul_v;
  var o1 = unwrap_or_zero(Some(d));
  var o2 = unwrap_or_zero(None);
  var div = safe_div(s, 5);
  if s == 45 && d == 29 && o1 == 29 && o2 == 0 && div == 9 { return 0; }
  return 1;
}
