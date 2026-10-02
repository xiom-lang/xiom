// Phase 3 checker gate, negative case: call arity mismatch.
fn add(a: Int, b: Int) -> Int { return a + b; }
fn main() -> Int {
  return add(1);
}
