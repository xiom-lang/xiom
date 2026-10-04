// Phase 3 checker gate, negative case: struct field access on a known type
// with no such field.
type Point = { x: Int; y: Int; }
fn f(p: Point) -> Int {
  return p.z;
}
