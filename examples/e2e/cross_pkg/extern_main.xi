// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module cross_pkg.extern_main
use extern_lib;
fn main() -> Int {
  return extern_lib.greet();
}
