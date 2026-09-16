// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_const_array
// BUG 25 #10 (crypto): const fixed-array element reads read the LENGTH slot
// (buf[0]) instead of the element -- the whole AES S-box lookup returned
// garbage. A const `[N]T = [...]` must read elements at index+1.

const _T: [4]UInt8 = [ 0x63, 0x7c, 0x77, 0x7b ];
const _BIG: [3]Int = [ 100, 200, 300 ];
const _E: [4]Int = [ 0x63, 0x7c, 0x77, 0x7b ];

fn main() -> Int {
  if _T[0] as Int != 0x63 { return 1; }
  if _T[1] as Int != 0x7c { return 2; }
  if _T[2] as Int != 0x77 { return 3; }
  if _T[3] as Int != 0x7b { return 4; }
  if _BIG[0] != 100 { return 5; }
  if _BIG[1] != 200 { return 6; }
  if _BIG[2] != 300 { return 7; }
  var i = 0;
  while i < 4 {
    if _T[i] as Int != _E[i] { return 10 + i; }
    i = i + 1;
  }
  return 0;
}
