// Phase 3 method gate: struct literal field type mismatch.
type Pt = { x: Int; }

fn main() -> Int {
  var p = Pt{ x: "s"; };
  return 0;
}
