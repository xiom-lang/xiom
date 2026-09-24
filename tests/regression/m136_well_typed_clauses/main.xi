// Relay lock (must stay green): well-typed clauses -- implicit-self calls,
// `@pre` snapshots and `result` -- keep compiling and running. The ensures
// below is the runtime-snapshot lock for `@pre` on a METHOD CALL (fixed
// 2026-09-24: the pre-state collector recorded the callee name instead of the
// receiver, so `len()@pre` read the post-call length).
module m136_well_typed_clauses;

pub type Box = { n: Int; }

pub fn Box.len(self) -> Int { return self.n; }

pub fn Box.bump(self)
  requires: self.n >= 0
  ensures: len() == len()@pre + 1
{
  self.n = self.n + 1;
}

fn double(x: Int) -> Int
  requires: x >= 0
  ensures: result == x + x
{
  return x + x;
}

fn main() -> Int {
  var b = Box{ n: 0 };
  b.bump();
  if b.len() != 1 { return 1; }
  if double(3) != 6 { return 2; }
  return 0;
}
