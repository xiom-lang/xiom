// Phase 3 pattern gate: a variant from another enum (m178 rule).
enum A { X(v: Int) }
enum B { Y(v: Int) }

fn f(a: A) -> Int {
  match a {
    B.Y(v) => { return v; }
    _ => { return 0; }
  }
}
