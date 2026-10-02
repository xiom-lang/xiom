// Phase 3 checker gate, warning case: W003 unreachable statement after an
// unconditional `return`.
fn main() -> Int {
  return 1;
  let x = 2;
  return x;
}
