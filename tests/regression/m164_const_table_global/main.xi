// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m164 lock: module-level const ARRAYS materialize as ONE `internal
// constant` global per table and index reads GEP it, instead of
// re-materializing the whole table on the stack at every use site
// (packages row 25: a 64K-entry table was rebuilt per hash). Semantics must
// match the old per-use buffer path bit-for-bit for Int/UInt8/Int8 element
// types (i64 slots).
module m164_const_table_global

const T: [8]Int = [10, 20, 30, 40, 50, 60, 70, 80];
const U: [3]UInt8 = [0xC0, 0x80, 0x01];
const S: [3]Int8 = [-1, -2, 5];

fn pick(i: Int) -> Int {
  return T[i];
}

fn main() -> Int {
  // Call-site read through a helper.
  if pick(2) != 30 { return 1; }

  // Loop accumulation (the old blowup shape: N+1 stores per use).
  var sum = 0;
  var i = 0;
  while i < 8 {
    sum = sum + T[i];
    i = i + 1;
  }
  if sum != 360 { return 2; }

  // UInt8 high-bit values must stay unsigned (zext parity).
  if U[0] as Int != 192 { return 3; }
  if U[1] as Int != 128 { return 4; }
  if U[2] as Int != 1 { return 5; }

  // Int8 negatives must round-trip through the i64 slot.
  if S[0] != -1 { return 6; }
  if S[1] != -2 { return 7; }
  if S[2] != 5 { return 8; }

  return 0;
}
