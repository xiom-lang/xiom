// M33-Y09: nested modules + enum + struct composition + match extraction + contract + impl + generic fn
type Item = { id: Int; weight: Int; }
enum Container { Single(val: Item), Pair(a: Item, b: Item) }
fn process[T](c: Container, op: Int) -> Int
  requires: op >= 0
  ensures: result >= 0
{
  match c {
    Single(val) => val.weight + op,
    Pair(a, b) => a.weight + b.weight + op,
  }
}
fn item_from(i: Int, w: Int) -> Item { return Item{ id: i; weight: w; }; }
interface Weighable { fn weight(self) -> Int; }
impl Weighable for Item {
  fn weight(self) -> Int { return self.weight; }
}
module core {
  pub fn do_process(c: Container, op: Int) -> Int { return process(c, op); }
  pub fn via_impl(i: Item) -> Int { return i.weight(); }
  module util {
    pub fn double(x: Int) -> Int { return x * 2; }
    pub fn square(x: Int) -> Int { return x * x; }
  }
}
use core.do_process;
use core.via_impl;
use core.util.double;
use core.util.square;
enum CallStyle { Process, Impl, Util }
fn calc(s: CallStyle, c: Container, op: Int, it: Item) -> Int {
  match s {
    Process => do_process(c, op),
    Impl => via_impl(it),
    Util => double(op),
  }
}
fn main() -> Int {
  var i1 = Item{ id: 1; weight: 5; };
  var i2 = Item{ id: 2; weight: 7; };
  var it = item_from(1, 3);
  var cont = Container.Pair(i1, i2);
  var r1 = calc(CallStyle.Process, cont, 3, it);
  var r2 = 5 + 7 + 3;
  var r3 = calc(CallStyle.Util, cont, 10, it);
  if r1 == r2 && r1 == 15 && r3 == 20 { return 0; }
  return 1;
}
