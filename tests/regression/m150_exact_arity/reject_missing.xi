// m150 (item 3, exact arity): missing arguments must be rejected -- the
// mirror hole of the extra-argument case. The e2e lock asserts this
// fixture FAILS to compile (the reject leg of the flip).
module m150_exact_arity_reject_missing;

fn f(a: Int, b: Int) -> Int { return a + b; }

fn main() -> Int {
  return f(1);
}
