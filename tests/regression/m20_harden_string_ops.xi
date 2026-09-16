// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use stdlib.xiom.string;
fn main() -> Int {
  var s = "hello";
  if string.str_len(s) != 5 { return 1; }
  var t = string.str_slice(s, 1, 4);
  if t != "ell" { return 2; }
  var u = string.str_slice(s, 0, 5);
  if u != s { return 3; }
  return 0;
}