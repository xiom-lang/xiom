// M32-X15: Combinatorial + Differential — all features: struct+enum+generic+match+contract
type Rect = { x: Int; y: Int; w: Int; h: Int; } derive[Eq]
enum Method { SumLoop, SumFormula, ProdLoop, ProdFormula }
fn sum_loop(lo: Int, hi: Int) -> Int
  requires: lo <= hi
  ensures: result >= 0
{
  var total: Int = 0;
  var i: Int = lo;
  while i <= hi { total = total + i; i = i + 1; }
  return total;
}
fn sum_formula(lo: Int, hi: Int) -> Int
  requires: lo <= hi
  ensures: result >= 0
{
  return (lo + hi) * (hi - lo + 1) / 2;
}
fn dispatch(m: Method, a: Int, b: Int) -> Int {
  match m {
    SumLoop => sum_loop(a, b),
    SumFormula => sum_formula(a, b),
  }
}
fn main() -> Int {
  var r = Rect{ x: 0; y: 0; w: 1; h: 10; };
  var s1 = dispatch(Method.SumLoop, r.x, r.h);
  var s2 = dispatch(Method.SumFormula, r.x, r.h);
  if s1 == s2 && s1 == 55 { return 0; }
  return 1;
}
