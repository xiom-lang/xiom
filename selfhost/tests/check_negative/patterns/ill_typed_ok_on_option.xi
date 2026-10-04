// Phase 3 pattern gate: `Ok` on an Option scrutinee (m178 rule).
fn f(o: Option[Int]) -> Int {
  match o {
    Ok(x) => { return x; }
    _ => { return 0; }
  }
}
