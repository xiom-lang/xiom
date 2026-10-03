// Phase 3 lint gate: W004 arms after an unguarded catch-all.
fn f(x: Int) -> Int {
  match x {
    _ => { return 0; }
    1 => { return 1; }
  }
}
