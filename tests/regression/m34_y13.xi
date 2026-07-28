// M34-Y13: derive Eq struct + generic + match + Option + compound assign + module
type Pair = { x: Int; y: Int; } derive[Eq]
enum BinOp { Add, Sub, Mul, Div, None }
fn eval[T](p: Pair, op: BinOp) -> Option[Int]
  requires: p.x >= 0
  requires: p.y >= 0
{
  match op {
    Add => { var r = p.x; r = r + p.y; return Some(r); }
    Sub => {
      var r = p.x;
      r = r - p.y;
      if r >= 0 { return Some(r); }
      else { return None; }
    }
    Mul => Some(p.x * p.y),
    Div => {
      if p.y == 0 { return None; }
      Some(p.x / p.y)
    }
    None => None,
  }
}
module calc_mod {
  pub fn calc(p: Pair, op: BinOp) -> Option[Int] { return eval(p, op); }
  pub fn eq_test(a: Pair, b: Pair) -> Bool { return a == b; }
}
use calc_mod.calc;
use calc_mod.eq_test;
interface BinaryOp { fn apply(self, op: BinOp) -> Option[Int]; }
impl BinaryOp for Pair {
  fn apply(self, op: BinOp) -> Option[Int] { return eval(self, op); }
}
fn main() -> Int {
  var p = Pair{ x: 7; y: 5; };
  var p2 = Pair{ x: 7; y: 5; };
  match calc(p, BinOp.Add) {
    Some(v) => { if v != 12 { return 1; } }
    None => { return 2; }
  }
  if !eq_test(p, p2) { return 3; }
  return 0;
}
