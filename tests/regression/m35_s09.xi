// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S09: Trim spaces -- remove leading and trailing spaces
use stdlib.xiom.string;
fn trim(s: Str) -> Str {
  var start: Int = 0;
  var end: Int = s.len();
  var space: UInt8 = 32;
  while start < end {
    if string.byte_at(s, start) == space { start = start + 1; }
    else { break; }
  }
  while end > start {
    if string.byte_at(s, end - 1) == space { end = end - 1; }
    else { break; }
  }
  return string.str_slice(s, start, end);
}
fn main() -> Int {
  if trim("  hello  ") == "hello" && trim("") == "" && trim("abc") == "abc" && trim("   ") == "" { return 0; }
  return 1;
}
