// m150 (item 3, exact arity): extra arguments must be rejected -- codegen
// used to silently drop them. The e2e lock asserts this fixture FAILS to
// compile (the reject leg of the flip).
module m150_exact_arity_reject_extra;

fn f(a: Int, b: Int) -> Int { return a + b; }

fn main() -> Int {
  return f(1, 2, 3);
}
