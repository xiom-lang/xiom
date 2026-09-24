// Relay lock (compile-FAIL): a clause-position Bool-vs-Int comparison must be
// rejected, not silently coerced. Asserted by
// crates/xiom/tests/checker_locks.rs; intentionally NOT in the e2e list.
module m135_clause_bool_int;

fn f(x: Int) -> Int
  requires: x == true
  ensures: result == false
{
  return x;
}

fn main() -> Int {
  return f(1);
}
