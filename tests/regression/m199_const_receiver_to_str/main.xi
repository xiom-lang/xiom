// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m199 (Pulse C-PULSE-05): method dispatch on a module-level `const` receiver
// must keep the const's concrete type. Pre-fix `SCHEMA_VERSION.to_str()` was
// classified as a module path (receiver dropped) and lowered to the
// erased-interface `@to_str()` typed stub (W005, empty render / abort).
use xiom.convert;

const VI: Int = 41;
const VS: Str = "abc";
const VF: Float64 = 1.5;
const VB: Bool = true;

fn main() -> Int {
  if VI.to_str() != "41" { return 1; }
  if VI.to_string() != "41" { return 2; }
  if VS.to_str() != "abc" { return 3; }
  if VB.to_str() != "true" { return 4; }
  if VF.to_str() != "1.5" { return 5; }
  if !VI.eq(41) { return 6; }
  if !VI.lt(100) { return 7; }
  return 0;
}
