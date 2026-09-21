// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E25: Multiple modules in one file -- use multiple imported modules
use stdlib.xiom.string;
fn main() -> Int {
  var s = "hello world";
  var len = string.str_len(s);
  if len != 11 { return 1; }
  return 0;
}
