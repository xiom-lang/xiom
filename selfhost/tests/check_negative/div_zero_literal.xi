// Phase 3 checker gate, warning case: W008 integer division by a zero
// literal always traps at runtime.
fn main() -> Int {
  var x = 10;
  return x / 0;
}
