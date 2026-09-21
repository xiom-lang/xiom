// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M14: Armstrong number (sum of digits^digit_count equals the number)
fn is_armstrong(n: Int) -> Int {
  if n < 0 { return 0; }
  var original = n;
  var digits: Int = 0;
  var temp = n;
  while temp > 0 {
    digits = digits + 1;
    temp = temp / 10;
  }
  var sum: Int = 0;
  temp = n;
  while temp > 0 {
    var d = temp % 10;
    var pow: Int = 1;
    var j: Int = 0;
    while j < digits {
      pow = pow * d;
      j = j + 1;
    }
    sum = sum + pow;
    temp = temp / 10;
  }
  if sum == original { return 1; }
  return 0;
}
fn main() -> Int {
  if is_armstrong(153) == 1 && is_armstrong(370) == 1 && is_armstrong(371) == 1 && is_armstrong(407) == 1 && is_armstrong(123) == 0 { return 0; }
  return 1;
}
