// Relay lock for the strict clause-mode transition switch: under the default
// (light) validator this non-Bool predicate compiles; with
// XIOM_STRICT_CLAUSES=1 the full predicate rule rejects it. Asserted by
// crates/xiom/tests/checker_locks.rs. Default stays light until the 8 stdlib
// clause sites are fixed (docs/COMPILER_BUGS.md, relay follow-ups).
module m137_clause_non_bool;

fn f(x: Int) -> Int
  requires: x
{
  return x;
}

fn main() -> Int {
  return f(1);
}
