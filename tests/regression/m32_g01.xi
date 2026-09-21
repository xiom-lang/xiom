// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-G01: Generic identity function
fn id[T](x: T) -> T { return x; }
fn main() -> Int {
  if id(42) != 42 { return 1; }
  if id(-1) != -1 { return 2; }
  return 0;
}
