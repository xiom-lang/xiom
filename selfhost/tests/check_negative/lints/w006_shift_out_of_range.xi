// Phase 3 lint gate: W006 shift amount out of range for the left type.
fn f(x: Int) -> Int {
  return x << 64;
}
