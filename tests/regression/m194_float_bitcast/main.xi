// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m194: `float_bits`/`bits_to_float` must reinterpret the IEEE-754 bit
// pattern exactly (LLVM bitcast). Pre-fix the stdlib fallbacks returned
// 0 / 0.0 (documented TODO(compiler)).
use xiom.num.float;
use xiom.io;

fn main() -> Int {
  // 1.5 == 0x3FF8000000000000.
  let bits = float.float_bits(1.5);
  if bits != 4609434218613702656 { return 1; }
  if float.bits_to_float(bits) != 1.5 { return 2; }

  // A negative value keeps its sign bit and round-trips.
  let neg = float.float_bits(-2.25);
  if neg >= 0 { return 3; }
  if float.bits_to_float(neg) != -2.25 { return 4; }
  if float.float_bits(float.bits_to_float(neg)) != neg { return 5; }

  // +0.0 is all-zero bits and round-trips.
  let zero = float.float_bits(0.0);
  if zero != 0 { return 6; }
  if float.bits_to_float(zero) != 0.0 { return 7; }

  io.println("m194 ok");
  return 0;
}
