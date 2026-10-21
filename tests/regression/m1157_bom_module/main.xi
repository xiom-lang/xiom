// m1157 (REL-20261021-1157-orbitdb): a UTF-8 BOM on a module file must not
// hide it from catalog discovery. `sub/impl.xi` intentionally starts with a
// BOM and declares a module path that does not match its file layout, so the
// catalog header index (not the entry-relative path lookup) must resolve it.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module repro.bom_main

use xiom.io;
use xiom.convert;
use repro.bom_impl;

pub fn main() -> Int {
  io.println("v=" + convert.int_to_string(bom_value()));
  if bom_value() != 7 { return 1; }
  return 0;
}
