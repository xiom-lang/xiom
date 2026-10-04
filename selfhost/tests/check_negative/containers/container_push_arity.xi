// Phase 3 container gate: builtin `Vec.push` arity is checked with Rust's
// arity-direct offset quirk (arg 1 maps to the self: Vec param).
fn main() -> Int {
  var v = Vec[Int].new();
  v.push(1, 2);
  return 0;
}
