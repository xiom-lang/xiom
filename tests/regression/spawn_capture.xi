// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use xiom.io;

fn main() -> Int {
  var x: Int = 100;
  spawn move {
    io.println("spawned");
    var result = x + 1;
    if result != 101 {
      io.println("FAIL");
    } else {
      io.println("PASS: spawn captured x=100");
    }
  }
  return 0;
}
