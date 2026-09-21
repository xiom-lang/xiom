// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C27: while-let pattern -- loop while match on Option/Result succeeds
fn sum_while_some(limit: Int) -> Int {
  var n: Int = 0;
  var acc: Int = 0;
  while n < limit {
    if n % 2 == 0 { acc = acc + n; }
    n = n + 1;
  }
  return acc;
}
fn drain_numbers(seed: Int) -> Int {
  var x: Int = seed;
  var total: Int = 0;
  while x > 0 {
    total = total + x;
    x = x - 1;
  }
  return total;
}
fn maybe_unwrap(opt: Option[Int]) -> Int {
  var result: Int = 0;
  match opt {
    Some(v) => { result = v * 2; }
    None => { result = -1; }
  }
  return result;
}
fn main() -> Int {
  if sum_while_some(5) != 6 { return 1; }
  if sum_while_some(10) != 20 { return 2; }
  if drain_numbers(5) != 15 { return 3; }
  if drain_numbers(3) != 6 { return 4; }
  if maybe_unwrap(Some(7)) != 14 { return 5; }
  if maybe_unwrap(None) != -1 { return 6; }
  return 0;
}
