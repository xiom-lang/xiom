// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-O13: ? with generic result -- ? on Result[T, E] with type param
fn ok_or_default[T](r: Result[T, Str], default: T) -> T {
  match r { Ok(v) => v, Err(_) => default }
}
fn get_pair(a: Int, b: Int) -> Result[Int, Str] {
  if a < 0 { return Err("negative a"); }
  if b < 0 { return Err("negative b"); }
  return Ok(a + b);
}
fn generic_chain(a: Int, b: Int, c: Int) -> Result[Int, Str] {
  var x = get_pair(a, b)?;
  var y = get_pair(x, c)?;
  return Ok(y);
}
fn main() -> Int {
  match generic_chain(1, 2, 3) { Ok(v) => { if v != 6 { return 1; } } Err(_) => { return 2; } }
  match generic_chain(-1, 2, 3) { Ok(_) => { return 3; } Err(_) => {} }
  var r: Result[Int, Str] = Ok(42);
  if ok_or_default(r, 0) != 42 { return 4; }
  var e: Result[Int, Str] = Err("fail");
  if ok_or_default(e, 99) != 99 { return 5; }
  return 0;
}
