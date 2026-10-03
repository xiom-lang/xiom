// Phase 3 pattern gate: `Some` on a Result scrutinee (m178 rule).
fn f(r: Result[Int, Str]) -> Int {
  match r {
    Some(x) => { return x; }
    _ => { return 0; }
  }
}
