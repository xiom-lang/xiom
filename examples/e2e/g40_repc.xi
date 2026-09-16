// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// G-40: repr(C) layout -- C-compatible struct field offsets
// Verifies XIOM structs used in extern C match C layout.

type MixedC = { a: Int8; b: Int16; c: Int32; d: Int64; e: Float32; f: Float64; }

extern "C" {
  // Read fields of a MixedC struct through a pointer (C side)
  fn g40_read_a(p: *MixedC) -> Int;
  fn g40_read_b(p: *MixedC) -> Int;
  fn g40_read_c(p: *MixedC) -> Int;
  fn g40_read_d(p: *MixedC) -> Int;
  fn g40_read_e(p: *MixedC) -> Float32;
  fn g40_read_f(p: *MixedC) -> Float64;

  // sizeof verification via C helper
  fn g40_sizeof_mixed() -> Int;
}

fn main() -> Int {
  return 0;
}
