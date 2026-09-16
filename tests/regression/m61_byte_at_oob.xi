// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M61 (stdlib report R1): byte_at upper out-of-range reads. xiom_byte_at
// only clamped pos < 0, so byte_at(s, pos >= len) read past the NUL
// terminator into adjacent heap bytes -- byte_at("", 999) returned
// whatever garbage neighbored the buffer (4 after a string-op preamble,
// 0 before one: heap-adjacency luck, not state). Fixed in the runtime:
// both xiom_byte_at and xiom_char_at now clamp pos >= strlen -> 0, making
// the result a pure function of (str, pos).
module m61_byte_at_oob

use xiom.string;
use xiom.io;

fn main() -> Int {
  // boundary positions on an empty string
  if xiom.string.byte_at("", 999) != 0u8 { return 1; }
  if xiom.string.byte_at("", 0) != 0u8 { return 2; }
  var s = "hello";
  // pos == len and beyond
  if xiom.string.byte_at(s, 5) != 0u8 { return 3; }
  if xiom.string.byte_at(s, 500) != 0u8 { return 4; }
  // negative
  if xiom.string.byte_at(s, -1) != 0u8 { return 5; }
  // in-bounds still works: 'o' == 111
  if xiom.string.byte_at(s, 4) != 111u8 { return 6; }
  // R1's exact contextual shape: case-mapping/slicing preamble first, then
  // the OOB access -- must STILL return 0 (pre-fix: garbage from adjacency).
  var up = xiom.string.str_upper("hello world");
  var low = xiom.string.str_lower(up);
  var sl = xiom.string.str_slice(low, 0, 5);
  if xiom.string.byte_at(sl, 500) != 0u8 { return 7; }
  if xiom.string.byte_at("", 999) != 0u8 { return 8; }
  return 0;
}
