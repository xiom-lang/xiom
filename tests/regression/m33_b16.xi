// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-B16: Shadowing with ownership -- shadowed let creates new binding, original inaccessible
fn main() -> Int {
  let a = 1;
  let a = a + 10;
  let a = a * 3;
  if a == 33 { return 0; }
  return 1;
}
