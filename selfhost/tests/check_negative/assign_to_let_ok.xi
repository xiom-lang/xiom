// Phase 3 checker gate, accept case: assignment to a `let` binding is NOT
// rejected by the Rust checker (no immutability diagnostic exists), so the
// selfhost port must stay silent too.
fn main() -> Int {
  let x = 1;
  x = 2;
  return x;
}
