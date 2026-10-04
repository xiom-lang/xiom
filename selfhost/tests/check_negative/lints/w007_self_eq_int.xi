// Phase 3 lint gate: W007 self-comparison on an Int is always true.
fn f(x: Int) -> Int {
  if x == x { return 1; }
  return 0;
}
