// M33-Y20: full combinatorial: generic struct+enum+impl+contract+match+method+module+diff
type Cell = { data: Int; flag: Int; } derive[Eq]
enum CellOp { Read, Write(v: Int), Toggle }
fn exec[T](c: Cell, op: CellOp) -> Int
  requires: c.data >= 0
  ensures: result >= 0
{
  match op {
    Read => c.data,
    Write(v) => v,
    Toggle => if c.flag == 0 { 1 } else { 0 },
  }
}
fn read_direct(c: Cell) -> Int { return c.data; }
fn toggle_direct(c: Cell) -> Int { return if c.flag == 0 { 1 } else { 0 }; }
interface CellOps { fn val(self) -> Int; fn toggled(self) -> Int; }
impl CellOps for Cell {
  fn val(self) -> Int { return self.data; }
  fn toggled(self) -> Int { return if self.flag == 0 { 1 } else { 0 }; }
}
module full {
  pub fn do_exec(c: Cell, op: CellOp) -> Int { return exec(c, op); }
  pub fn do_read(c: Cell) -> Int { return read_direct(c); }
  pub fn do_toggle(c: Cell) -> Int { return toggle_direct(c); }
  pub fn via_val(c: Cell) -> Int { return c.val(); }
  pub fn via_toggle(c: Cell) -> Int { return c.toggled(); }
}
use full.do_exec;
use full.do_read;
use full.do_toggle;
use full.via_val;
use full.via_toggle;
enum Way { Exec, DirectRead, DirectToggle, ImplVal, ImplToggle }
fn dispatch(w: Way, c: Cell, op: CellOp) -> Int {
  match w {
    Exec => do_exec(c, op),
    DirectRead => do_read(c),
    DirectToggle => do_toggle(c),
    ImplVal => via_val(c),
    ImplToggle => via_toggle(c),
  }
}
fn main() -> Int {
  var c = Cell{ data: 99; flag: 0; };
  var r1 = dispatch(Way.Exec, c, CellOp.Read);
  var r2 = dispatch(Way.DirectRead, c, CellOp.Read);
  var r3 = dispatch(Way.ImplVal, c, CellOp.Read);
  var r4 = dispatch(Way.Exec, c, CellOp.Toggle);
  var r5 = dispatch(Way.DirectToggle, c, CellOp.Toggle);
  var r6 = dispatch(Way.ImplToggle, c, CellOp.Toggle);
  if r1 == r2 && r2 == r3 && r1 == 99 { return 0; }
  if r4 == r5 && r5 == r6 && r4 == 1 { return 0; }
  return 1;
}
