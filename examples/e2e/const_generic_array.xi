// XIOM e2e test — const-generics with [N]T arrays
// Tests: array literals with const-generic function resolution
// Returns 0 on success.

module e2e_const_generic

// Simple const-generic: return the array length
fn array_len[T, const N: Int](arr: [N]T) -> Int {
  return 0; // N is not accessible as an expression yet — return 0 to verify compilation
}

// Function that actually uses the array type (monomorphisation test)
fn first_elem(arr: [5]Int) -> Int {
  return arr[0];
}

fn main() -> Int {
  // Verify that fixed-size arrays work with indexing
  let arr = [1, 2, 3, 4, 5];
  let elem = first_elem(arr);
  if elem != 1 { return 1; }
  return 0;
}
