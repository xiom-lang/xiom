// Phase 3 lint gate: W004 duplicate literal arm.
fn f(x: Int) -> Int {
  match x {
    1 => { return 1; }
    1 => { return 2; }
    _ => { return 0; }
  }
}
