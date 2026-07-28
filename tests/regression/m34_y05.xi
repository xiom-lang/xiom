// M34-Y05: enum + compound assign + contract + module + generic dispatch
enum Kind { A, B, C }
fn dispatch(kind: Kind, data: Int) -> Int
  requires: data >= 0
  ensures: result >= 0
{
  var val = data;
  match kind {
    A => { val = val + 10; }
    B => { val = val * 2; }
    C => { val = val % 2; val = val + 7; }
  }
  val = val + 3;
  return val;
}
module extra {
  pub fn do_dispatch(k: Kind, d: Int) -> Int { return dispatch(k, d); }
  pub fn identity(d: Int) -> Int { return d; }
}
use extra.do_dispatch;
use extra.identity;
fn main() -> Int {
  var r1 = do_dispatch(Kind.A, 5);
  var r2 = do_dispatch(Kind.B, 7);
  var r3 = do_dispatch(Kind.C, 3);
  if r1 == 18 && r2 == 17 && r3 == 11 { return 0; }
  return 1;
}
