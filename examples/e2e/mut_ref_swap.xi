// E2E: generic swap via &mut scalar references
// Verifies that &mut Int works in generic functions (ARC A + B).
// Returns 0 on success.

module e2e_mut_ref_swap

fn swap[T](a: &mut T, b: &mut T) {
  let temp = *a;
  *a = *b;
  *b = temp;
}

fn main() -> Int {
  var x = 10;
  var y = 20;
  swap(&mut x, &mut y);
  if x == 20 && y == 10 { return 0; }
  return 1;
}
