// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn fizzbuzz(n: Int) -> Str {
  if n % 15 == 0 { return "FizzBuzz"; }
  if n % 3 == 0 { return "Fizz"; }
  if n % 5 == 0 { return "Buzz"; }
  return n.to_string();
}
fn main() -> Int {
  if fizzbuzz(15) != "FizzBuzz" { return 1; }
  if fizzbuzz(3) != "Fizz" { return 2; }
  if fizzbuzz(5) != "Buzz" { return 3; }
  if fizzbuzz(7) != "7" { return 4; }
  return 0;
}