// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M23: pi approximation via Leibniz series: pi/4 = 1 - 1/3 + 1/5 - 1/7 + ...
fn leibniz_pi(terms: Int) -> Float64 {
  var sum: Float64 = 0.0;
  var sign: Float64 = 1.0;
  var i: Int = 0;
  while i < terms {
    var denom: Float64 = (2 * i + 1) as Float64;
    sum = sum + sign / denom;
    sign = 0.0 - sign;
    i = i + 1;
  }
  return sum * 4.0;
}
fn main() -> Int {
  var pi_approx: Float64 = leibniz_pi(10000);
  if pi_approx > 3.14 && pi_approx < 3.15 { return 0; }
  return 1;
}
