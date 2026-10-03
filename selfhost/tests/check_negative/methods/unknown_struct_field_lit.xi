// Phase 3 method gate: struct literal with an unknown field.
type Pt = { x: Int; y: Int; }

fn main() -> Int {
  var p = Pt{ x: 1; z: 2; };
  return 0;
}
