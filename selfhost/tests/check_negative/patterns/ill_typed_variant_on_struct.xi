// Phase 3 pattern gate: a variant pattern on a struct scrutinee.
type P = { x: Int; }

fn f(p: P) -> Int {
  match p {
    Q.Y(v) => { return v; }
    _ => { return 0; }
  }
}
