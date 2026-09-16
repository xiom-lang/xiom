// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-V08: Int max as Float64, Float64 to Int truncation
fn main() -> Int {
  var big_int: Int = 2147483647;
  var as_float: Float64 = big_int as Float64;
  var back_int: Int = as_float as Int;
  var trunc: Float64 = 99.9;
  var trunc_int: Int = trunc as Int;
  if back_int == big_int && trunc_int == 99 {
    return 0;
  }
  return 1;
}
