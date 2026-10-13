// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m298 "tally" module (xvector role): imports demo.metrics and declares
// `hits: Counter`, meaning the PACKAGE Counter. Pre-fix the bare reference
// was never rewritten at injection, so codegen's deterministic same-leaf
// pick chose demo.shim.Counter (shorter key) for the field layout while the
// initializer's metric_counter_new() returned demo.metrics.Counter -- clang
// store mismatch. Post-fix the unique imported owner binds the field to
// demo.metrics.Counter.
module demo.tally;

use demo.metrics;

pub type Tally = {
  hits: Counter;
}

pub fn tally_new() -> Tally {
  return Tally{ hits: metric_counter_new() };
}

pub fn tally_hit(t: &mut Tally) {
  metric_counter_inc(&mut t.hits);
}

pub fn tally_hits(t: &Tally) -> Int {
  return metric_counter_value(&t.hits);
}
