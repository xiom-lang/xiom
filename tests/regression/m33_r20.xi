// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn is_ok_and(r: Result[Int, Str], pred: fn(Int) -> Bool) -> Bool {
  match r { Ok(v) => pred(v), Err(_) => false }
}
fn positive(x: Int) -> Bool { return x > 0; }
fn gt10(x: Int) -> Bool { return x > 10; }
fn lt200(x: Int) -> Bool { return x < 200; }
fn main() -> Int {
  if !is_ok_and(Ok(42), positive) { return 1; }
  if is_ok_and(Ok(5), gt10) { return 2; }
  if is_ok_and(Err("fail"), positive) { return 3; }
  if !is_ok_and(Ok(100), lt200) { return 4; }
  var bad: Result[Int, Str] = Err("x");
  if !bad.is_err() { return 5; }
  return 0;
}
