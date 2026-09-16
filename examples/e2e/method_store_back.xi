// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// E2E: struct method store-back-to-receiver
// Verifies that calling a mutating method on a struct var persists the
// mutation via automatic store-back (fixed in Cluster 2). Returns 0 on success.

module e2e_method_store_back

type Counter = { value: Int; }

fn Counter.inc(self) -> Counter {
  self.value = self.value + 1;
  self
}

fn main() -> Int {
  var c = Counter { value: 41; };
  c.inc();
  if c.value == 42 {
    return 0;
  }
  return 1;
}
