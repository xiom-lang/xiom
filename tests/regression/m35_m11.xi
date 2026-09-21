// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M11: fibonacci (iterative)
fn fib(n: Int) -> Int {
  if n <= 0 { return 0; }
  if n == 1 { return 1; }
  var a: Int = 0;
  var b: Int = 1;
  var i: Int = 2;
  while i <= n {
    var c = a + b;
    a = b;
    b = c;
    i = i + 1;
  }
  return b;
}
fn main() -> Int {
  if fib(0) == 0 && fib(1) == 1 && fib(2) == 1 && fib(3) == 2 && fib(10) == 55 && fib(12) == 144 { return 0; }
  return 1;
}
