// M33-Y15: generic dispatch table via enum + struct data + match + contract + impl + module + diff
type Entry = { key: Int; value: Int; }
enum TableOp { Lookup, Insert(key: Int, val: Int), Delete }
fn operate[T](e: Entry, op: TableOp) -> Int
  requires: e.key >= 0
  ensures: result >= 0
{
  match op {
    Lookup => e.value,
    Insert(key, val) => if e.key == key { val } else { e.value },
    Delete => 0,
  }
}
fn lookup_direct(e: Entry) -> Int { return e.value; }
interface TableOps { fn read(self) -> Int; }
impl TableOps for Entry {
  fn read(self) -> Int { return self.value; }
}
module table {
  pub fn do_operate(e: Entry, op: TableOp) -> Int { return operate(e, op); }
  pub fn do_lookup(e: Entry) -> Int { return lookup_direct(e); }
  pub fn via_read(e: Entry) -> Int { return e.read(); }
}
use table.do_operate;
use table.do_lookup;
use table.via_read;
enum Way { Op, Direct, Read }
fn access(w: Way, e: Entry, op: TableOp) -> Int {
  match w { Op => do_operate(e, op), Direct => do_lookup(e), Read => via_read(e), }
}
fn main() -> Int {
  var e = Entry{ key: 1; value: 42; };
  var r1 = access(Way.Op, e, TableOp.Lookup);
  var r2 = access(Way.Direct, e, TableOp.Lookup);
  var r3 = access(Way.Read, e, TableOp.Lookup);
  if r1 == r2 && r2 == r3 && r1 == 42 { return 0; }
  return 1;
}
