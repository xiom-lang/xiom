// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m298 "metrics package" module (xvector role): exports its own Counter and
// the metric_counter_* API a consumer module imports.
module demo.metrics;

pub type Counter = {
  value: Int;
}

pub fn metric_counter_new() -> Counter {
  return Counter{ value: 0 };
}

pub fn metric_counter_inc(c: &mut Counter) {
  c.value = c.value + 1;
}

pub fn metric_counter_value(c: &Counter) -> Int {
  return c.value;
}
