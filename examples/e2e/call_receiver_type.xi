// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// E2E: call expression receiver type inference
// Verifies that `call().method()` resolves the method on the returned
// struct type (fixed in Cluster 1b/iter). Returns 0 on success.

module e2e_call_receiver

type Pair = { a: Int; b: Int; }

fn make_pair(x: Int, y: Int) -> Pair {
  Pair { a: x; b: y; }
}

fn Pair.sum(self) -> Int {
  self.a + self.b
}

fn main() -> Int {
  let s = make_pair(10, 25).sum();
  if s == 35 {
    return 0;
  }
  return 1;
}
