// M33-Y05: generic dispatch fn + struct derive + contract chain + match + enum + impl + module + diff
type Triple = { a: Int; b: Int; c: Int; } derive[Eq]
enum BaseOp { Add, Sub }
enum MetaOp { Combine, ChooseMax }
fn apply_op[T](t: Triple, op: BaseOp) -> Int
  requires: t.a >= 0
  requires: t.b >= 0
  ensures: result >= 0
{
  match op {
    BaseOp.Add => t.a + t.b,
    BaseOp.Sub => if t.a > t.b { t.a - t.b } else { t.b - t.a },
  }
}
fn meta[T](t: Triple, m: MetaOp) -> Int
  requires: t.c >= 0
  ensures: result >= 0
{
  match m {
    Combine => apply_op(t, BaseOp.Add) + t.c,
    ChooseMax => { var sum = apply_op(t, BaseOp.Add); if sum > t.c { sum } else { t.c } },
  }
}
interface TripleOps { fn sum(self) -> Int; }
impl TripleOps for Triple {
  fn sum(self) -> Int { return self.a + self.b + self.c; }
}
module calc {
  pub fn op_add(t: Triple) -> Int { return apply_op(t, BaseOp.Add); }
  pub fn meta_combine(t: Triple) -> Int { return meta(t, MetaOp.Combine); }
  pub fn via_iface(t: Triple) -> Int { return t.sum(); }
}
use calc.op_add;
use calc.meta_combine;
use calc.via_iface;
enum Strategy { Direct, MetaOp, Interface }
fn strategy(s: Strategy, t: Triple) -> Int {
  match s { Direct => op_add(t), MetaOp => meta_combine(t), Interface => via_iface(t), }
}
fn main() -> Int {
  var t = Triple{ a: 5; b: 10; c: 3; };
  var r1 = strategy(Strategy.Direct, t);
  var r2 = strategy(Strategy.Interface, t);
  if r1 == 15 && r2 == 18 { return 0; }
  return 1;
}
