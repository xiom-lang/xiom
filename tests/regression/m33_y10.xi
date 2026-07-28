// M33-Y10: multi-interface impl + generic struct + enum match + contract chain + module + diff
type Data = { val: Int; }
enum OpKind { Inc, Dec, Identity, Double }
fn apply[T](d: Data, op: OpKind) -> Int
  requires: d.val >= 0
  ensures: result >= 0
{
  match op {
    Inc => d.val + 1,
    Dec => if d.val > 0 { d.val - 1 } else { 0 },
    Identity => d.val,
    Double => d.val * 2,
  }
}
fn inc_twice(d: Data) -> Int { return d.val + 2; }
interface Mappable { fn map(self) -> Int; }
interface Scalable { fn scale(self, factor: Int) -> Int; }
impl Mappable for Data {
  fn map(self) -> Int { return self.val * 2; }
}
impl Scalable for Data {
  fn scale(self, factor: Int) -> Int { return self.val * factor; }
}
module algebra {
  pub fn do_apply(d: Data, op: OpKind) -> Int { return apply(d, op); }
  pub fn do_inc2(d: Data) -> Int { return inc_twice(d); }
  pub fn via_map(d: Data) -> Int { return d.map(); }
  pub fn via_scale(d: Data, f: Int) -> Int { return d.scale(f); }
}
use algebra.do_apply;
use algebra.do_inc2;
use algebra.via_map;
use algebra.via_scale;
enum Algo { Direct, Inc2, Map, Scale }
fn run(algo: Algo, d: Data, op: OpKind, f: Int) -> Int {
  match algo {
    Direct => do_apply(d, op),
    Inc2 => do_inc2(d),
    Map => via_map(d),
    Scale => via_scale(d, f),
  }
}
fn main() -> Int {
  var d = Data{ val: 7; };
  var r1 = run(Algo.Direct, d, OpKind.Double, 2);
  var r2 = run(Algo.Map, d, OpKind.Double, 2);
  var r3 = run(Algo.Scale, d, OpKind.Double, 3);
  if r1 == r2 && r1 == 14 && r3 == 21 { return 0; }
  return 1;
}
