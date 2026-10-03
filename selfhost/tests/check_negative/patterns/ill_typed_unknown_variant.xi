// Phase 3 pattern gate: an unknown variant on an enum scrutinee.
enum A { X(v: Int) }

fn f(a: A) -> Int {
  match a {
    Nope(v) => { return v; }
    _ => { return 0; }
  }
}
