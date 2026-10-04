// Phase 3 method gate, accept case: primitive builtin methods (Str.len).
fn main() -> Int {
  var s = "ab";
  if s.len() != 2 { return 1; }
  return 0;
}
