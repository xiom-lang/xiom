// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m298 "shim" module (xvector role): its own Counter shares only the short
// name with demo.metrics' Counter. Shortest same-leaf key, so pre-fix it
// won every bare-Counter suffix pick.
module demo.shim;

pub type Counter = {
  name: Str;
  value: Int;
}

pub fn counter_new(name: Str) -> Counter {
  return Counter{ name: name; value: 0 };
}

pub fn counter_inc(c: &mut Counter) {
  c.value = c.value + 1;
}
