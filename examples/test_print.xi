// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module test_print
use xiom.io;
use xiom.math;
fn main() -> Int {
  var x: Float64 = 3.14159;
  io.println(x.to_str());
  io.println(math.sqrt(4.0).to_str());
  return 0;
}
