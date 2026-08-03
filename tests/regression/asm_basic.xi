// Inline Assembly Tests — v0.55
// Verifies asm("nop") compiles and runs correctly.
// Returns 0 on success.

fn main() -> Int {
  // Basic: nop — does nothing, just verifies asm compiles and links
  asm("nop");

  // Verify return reaches here
  return 0;
}
