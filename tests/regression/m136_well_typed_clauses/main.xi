// Relay lock (must stay green): well-typed clauses -- implicit-self calls,
// `@pre` snapshots and `result` -- keep compiling and running after the
// clause Bool-mix rejection landed.
//
// NOTE: `@pre` on a METHOD CALL (`len()@pre`) is exercised for its TYPING
// only (`>= 0` holds whatever the runtime snapshot returns); the runtime
// snapshot for call-position @pre is a separate open finding
// (docs/COMPILER_BUGS.md, relay follow-up).
module m136_well_typed_clauses;

pub type Box = { n: Int; }

pub fn Box.len(self) -> Int { return self.n; }

pub fn Box.bump(self)
  requires: self.n >= 0
  ensures: len()@pre >= 0
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
