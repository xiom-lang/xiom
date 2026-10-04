// Phase 3 checker gate, accept case: unknown type annotations pass (legacy
// behavior -- the Rust checker only records them; no diagnostic).
fn f(x: NoSuchType) -> Int {
  return 0;
}
