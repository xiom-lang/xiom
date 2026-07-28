// M33-Y06: interface+impl + generic enum + struct composition + match + contract + module + diff
type Box = { w: Int; h: Int; d: Int; }
type LabeledBox = { b: Box; tag: Int; }
enum OpKind { Volume, Surface, Tag }
fn operate[T](lb: LabeledBox, op: OpKind) -> Int
  requires: lb.b.w >= 0
  requires: lb.b.h >= 0
  requires: lb.b.d >= 0
  ensures: result >= 0
{
  match op {
    Volume => lb.b.w * lb.b.h * lb.b.d,
    Surface => 2 * (lb.b.w * lb.b.h + lb.b.h * lb.b.d + lb.b.w * lb.b.d),
    Tag => lb.tag,
  }
}
interface Volumed { fn volume(self) -> Int; }
impl Volumed for Box {
  fn volume(self) -> Int { return self.w * self.h * self.d; }
}
module engine {
  pub fn op_volume(lb: LabeledBox) -> Int { return operate(lb, OpKind.Volume); }
  pub fn op_tag(lb: LabeledBox) -> Int { return operate(lb, Tag); }
  pub fn via_iface(b: Box) -> Int { return b.volume(); }
}
use engine.op_volume;
use engine.op_tag;
use engine.via_iface;
enum ComputeVia { Op, Iface }
fn compute(c: ComputeVia, lb: LabeledBox) -> Int {
  match c { Op => op_volume(lb), Iface => via_iface(lb.b), }
}
fn main() -> Int {
  var b = Box{ w: 2; h: 3; d: 4; };
  var lb = LabeledBox{ b: b; tag: 99; };
  var r1 = compute(ComputeVia.Op, lb);
  var r2 = compute(ComputeVia.Iface, lb);
  if r1 == r2 && r1 == 24 { return 0; }
  return 1;
}
