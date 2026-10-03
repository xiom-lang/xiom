// Phase 3 container gate, accept case: builtin `Vec.push` accepts a generic
// payload (T is skipped in the argument check).
fn main() -> Int {
  var v = Vec[Int].new();
  v.push(1);
  return 0;
}
