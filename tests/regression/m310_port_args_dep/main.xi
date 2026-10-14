// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m310 lock (REL-20261010-1549-packages-5): the dependency `helper` ships
// port.args.json naming its C source; WITHOUT --c-source on the consumer
// command line the pre-fix link failed with an undefined symbol for the
// extern below (the dependency's build args were never applied).
module m310_port_args_dep.main

use m310_port_args_helper.helper;

fn main() -> Int {
  let v = helper.bump(1);
  if v != 42 { return 1; }
  return 0;
}
