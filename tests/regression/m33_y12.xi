// M33-Y12: higher-order enum + generic wrapper struct + match + contract cascading + impl + module + diff
type Wrapper = { data: Int; id: Int; } derive[Eq]
enum Action { Set(val: Int), Add(v: Int), Scale(f: Int) }
fn run[T](w: Wrapper, a: Action) -> Int
  requires: w.data >= 0
  ensures: result >= 0
{
  match a {
    Set(val) => val,
    Add(v) => w.data + v,
    Scale(f) => w.data * f,
  }
}
fn add_ten(w: Wrapper) -> Int { return w.data + 10; }
interface Processable { fn proc(self) -> Int; }
impl Processable for Wrapper {
  fn proc(self) -> Int { return self.data + self.id; }
}
module wrap {
  pub fn do_run(w: Wrapper, a: Action) -> Int { return run(w, a); }
  pub fn do_ten(w: Wrapper) -> Int { return add_ten(w); }
  pub fn via_proc(w: Wrapper) -> Int { return w.proc(); }
}
use wrap.do_run;
use wrap.do_ten;
use wrap.via_proc;
enum Way { Run, Ten, Proc }
fn execute(w: Way, wr: Wrapper, act: Action) -> Int {
  match w { Run => do_run(wr, act), Ten => do_ten(wr), Proc => via_proc(wr), }
}
fn main() -> Int {
  var w = Wrapper{ data: 5; id: 3; };
  var r1 = execute(Way.Run, w, Action.Add(2));
  var r2 = execute(Way.Run, w, Action.Add(2));
  var r3 = execute(Way.Proc, w, Action.Add(0));
  if r1 == 7 && r2 == 7 && r3 == 8 { return 0; }
  return 1;
}
