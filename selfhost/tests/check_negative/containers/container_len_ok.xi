// Phase 3 container gate, accept case: `Vec.len` returns Int.
fn main() -> Int {
  var v = Vec[Int].new();
  v.push(1); v.push(2);
  if v.len() != 2 { return 1; }
  return 0;
}
