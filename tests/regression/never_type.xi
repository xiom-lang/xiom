// Never Type Tests — v0.55
// Verifies ! (never) type parsing and basic compilation.
// Returns 0 on success.

// Function declaring never-return type
fn wont_return() -> ! {
  // Infinite loop — never returns
  while true { }
}

fn main() -> Int {
  // Can store result of !-typed expr (but it never executes)
  var ok: Bool = true;
  if ok {
    return 0;
  }
  // This is unreachable
  wont_return();
  return 1;
}
