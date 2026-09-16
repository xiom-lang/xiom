// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C06: while true loop -- infinite loop with internal break
fn main() -> Int {
  var count: Int = 0;
  while true {
    count = count + 1;
    if count >= 10 { break; }
  }
  if count != 10 { return 1; }
  return 0;
}
