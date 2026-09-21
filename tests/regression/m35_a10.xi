// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-A10: Fibonacci -- iterative implementation using while loop
fn fib_iter(n: Int) -> Int {
  if n <= 1 { return n; }
  var a: Int = 0;
  var b: Int = 1;
  var i: Int = 1;
  while i < n {
    var t: Int = a + b;
    a = b;
    b = t;
    i = i + 1;
  }
  return b;
}
fn main() -> Int {
  if fib_iter(0) != 0 { return 1; }
  if fib_iter(1) != 1 { return 2; }
  if fib_iter(10) != 55 { return 3; }
  if fib_iter(20) != 6765 { return 4; }
  return 0;
}
