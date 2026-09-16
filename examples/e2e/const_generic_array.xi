// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM e2e test -- const-generics with [N]T arrays
// Tests array indexing on let-bound and literal arrays (5a.7)
// + const-declared loop size (5a.5)
// Returns 0 on success.

module e2e_const_generic

fn main() -> Int {
  // Test: let-bound array indexing
  let arr = [10, 20, 30, 40, 50];
  let a0 = arr[0];
  let a4 = arr[4];
  if a0 != 10 { return 1; }
  if a4 != 50 { return 2; }

  // Test: const-declared size used in while loop
  const N: Int = 5;
  var arr2 = [1, 2, 3, 4, 5];
  var sum = 0;
  var i = 0;
  while i < N {
    sum = sum + arr2[i];
    i = i + 1;
  }
  if sum != 15 { return 3; }

  return 0;
}
