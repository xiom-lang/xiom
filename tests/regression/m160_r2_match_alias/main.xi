// m160 (R-2 partial): an INLINE aggregate Option payload (concrete
// Option__Counter layout) binds by address, so field writes and method
// calls through `Some(c)` persist (r2_match_mutation printed 6/6 before;
// 6/7 expected). Call-result scrutinees keep the snapshot path and must
// still work.
module m160_r2_match_alias;

use xiom.io;

type Counter = { n: Int; }

fn Counter.inc(self) -> Int {
  self.n = self.n + 1;
  return self.n;
}

fn make() -> Option[Counter] {
  return Some(Counter { n: 1 });
}

fn main() -> Int {
  var opt: Option[Counter] = Some(Counter { n: 5 });
  match opt {
    Some(c) => { if c.inc() != 6 { return 1; } }
    None => { return 2; }
  }
  match opt {
    Some(c) => {
      if c.n != 6 { return 3; }
      if c.inc() != 7 { return 4; }
    }
    None => { return 5; }
  }

  match make() {
    Some(c) => { if c.inc() != 2 { return 6; } }
    None => { return 7; }
  }

  io.println("ok");
  return 0;
}
