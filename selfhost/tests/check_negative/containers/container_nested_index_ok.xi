// Phase 3 container gate, accept case: constructor results keep their type
// arguments so nested indexing yields the element type.
fn main() -> Int {
  var m = Vec[Vec[Int]].new();
  var r = Vec[Int].new();
  r.push(7);
  m.push(r);
  if m[0][0] != 7 { return 1; }
  return 0;
}
