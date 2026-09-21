// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m75 shim: same module LEAF ("base32") and same fn names as m75canon.base32,
// delegating through a `use ... as` ALIAS. R20: the emitter must bind these
// calls to the checker-recorded owner-qualified target; the old span-only
// lookup could not see the alias and fell through to an order-dependent
// `.name` suffix scan that sometimes bound this shim itself (infinite
// recursion -> 0xC0000005) or a zero-arg stub (empty Str / lost Result).
module m75conv.base32

use m75canon.base32 as enc32;

pub fn encode(x: Int) -> Int {
  return enc32.encode(x);
}

pub fn name() -> Str {
  return enc32.name();
}

pub fn decode(x: Int) -> Result[Int, Str] {
  return enc32.decode(x);
}
