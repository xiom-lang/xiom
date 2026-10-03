// Phase 3 method gate: unknown method on a user struct receiver.
type Pt = { x: Int; }

fn f(p: Pt) -> Int {
  return p.nope();
}
