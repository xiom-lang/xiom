// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// E2E regression: enum equality (==) and match on a value receiver.
// Locks in the `.eq` fallback for builtin-style enums (compare field 0 inline)
// and enum-variant construction as values. Returns 0 on success.
module e2e_enum_eq

type Color = enum { Red, Green, Blue }

fn pick(n: Int) -> Color {
  match n {
    0 => Red,
    1 => Green,
    _ => Blue,
  }
}

fn main() -> Int {
  let a = pick(1);       // Green
  let b = pick(1);       // Green
  let c = pick(2);       // Blue
  // Enum == enum via derived/builtin eq fallback; != too.
  if a == b && a != c {
    return 0;
  }
  return 1;
}
