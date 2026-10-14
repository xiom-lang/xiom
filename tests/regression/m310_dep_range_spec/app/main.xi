// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m310 lock (REL-20261010-1549-packages-7): a `>=0.2.0` dependency spec
// must resolve the installed xiom-rate-0.2.0 (fixture XIOM_HOME).
module m310_dep_range_spec.app

use xiom.rate;

fn main() -> Int {
  let r = rate.rate_new();
  if r != 1 { return 1; }
  return 0;
}
