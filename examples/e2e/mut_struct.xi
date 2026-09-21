// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// E2E: &mut Struct pointer passing -- mutations propagate to caller
// Verifies ARC C: &mut Struct parameters are real pointers.
// Returns 0 on success.

module e2e_mut_struct

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
