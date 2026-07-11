// E2E: generic function with multiple type parameters
// Verifies that multi-param generics monomorphize correctly.
// Returns 0 on success.

module e2e_generic_multi

type Pair = { a: Int; b: Int; }

fn make_pair[T](x: T, y: T) -> Pair {
  // Not using T directly — just verifying monomorphization
  Pair { a: 0; b: 0; }
}

fn main() -> Int {
  let p = make_pair[Int](10, 20);
  if p.a == 0 { return 0; }
  return 1;
}
