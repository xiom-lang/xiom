// REL-20261024-1101-pulse fixture: arm io.print must emit the variadic
// @printf call-site type. LLVM 14/clang on arm64 rejected the fixed
// two-argument form that the stdlib extern spelling produced.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module arm_io_print_printf

use xiom.io;

fn main() -> Int {
  let s = "hello";
  io.print(s);
  return 0;
}
