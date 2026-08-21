// M32-X02: Combinatorial + Differential -- loop sum vs formula with struct+enum
type Params = { lo: Int; hi: Int; }
enum CalcMode { SumLoop, SumFormula }
fn sum_loop(p: Params) -> Int {
  var total: Int = 0;
  var i: Int = p.lo;
  while i <= p.hi { total = total + i; i = i + 1; }
  return total;
}
fn sum_formula(lo: Int, hi: Int) -> Int {
  return (lo + hi) * (hi - lo + 1) / 2;
}
fn select(mode: CalcMode, p: Params) -> Int {
  match mode {
    SumLoop => sum_loop(p),
    SumFormula => sum_formula(p.lo, p.hi),
  }
}
fn main() -> Int {
  var p = Params{ lo: 1; hi: 100; };
  var a = select(CalcMode.SumLoop, p);
  var b = select(CalcMode.SumFormula, p);
  if a == b && a == 5050 { return 0; }
  return 1;
}
