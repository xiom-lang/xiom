// Phase 3 lint gate: W007 self-comparison on a Str is always false.
fn f(s: Str) -> Int {
  if s != s { return 1; }
  return 0;
}
