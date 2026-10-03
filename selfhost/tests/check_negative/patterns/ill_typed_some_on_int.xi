// Phase 3 pattern gate: `Some` on an Int scrutinee (m178 rule).
fn f(i: Int) -> Int {
  match i {
    Some(x) => { return x; }
    _ => { return 0; }
  }
}
