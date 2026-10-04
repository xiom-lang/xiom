// Phase 3 checker gate, accept case: a match in one function must not leak
// scope state into the next function (regression for the scope-stack
// live-count bug found while porting the checker).
fn f(o: Option[Int]) -> Int {
  match o {
    Some(x) => { return x; }
  }
  return 0;
}
fn g(b: Bool) -> Int {
  match b {
    true => { return 1; }
  }
  return 0;
}
