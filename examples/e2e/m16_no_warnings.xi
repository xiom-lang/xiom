// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M16: Hello World compiles with ZERO warnings (was 5 warnings before fix)
// Regression test for: Vec[UInt8], generic T, Self warnings
use xiom.io;

fn main() {
  io.println("Hello, XIOM!");
}
