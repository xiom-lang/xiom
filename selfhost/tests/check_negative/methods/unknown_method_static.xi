// Phase 3 method gate: unknown static method on a known type.
type Pt = { x: Int; }

fn main() -> Int {
  var y = Pt.nope(1);
  return 0;
}
