// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m198 (Pulse C-PULSE-06): an incomplete struct literal must be a compile
// error. Pre-fix `Pair{ a: 1; }` compiled and `p.b.len()` read garbage
// (uninitialized Vec header); PULSE's server AV'd on real requests.
type Pair = { a: Int; b: Vec[UInt8]; }

fn main() -> Int {
  let p = Pair{ a: 1; };
  return p.a;
}
