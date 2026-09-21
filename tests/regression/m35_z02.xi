// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z02: generic+Option+Result+type_alias+contract+match+while+array+compound_assign
type Code = Int;
type Boxed = { val: Int; ok: Bool; }
enum Status { Ok(val: Int), Fail(reason: Str) }
fn validate[T](v: Int, min: Int) -> Option[Int]
  requires: min >= 0
{
  if v >= min { return Some(v); }
  return None;
}
fn combine(o1: Option[Int], o2: Option[Int]) -> Result[Int, Str] {
  match o1 { Some(a) => { match o2 { Some(b) => Ok(a + b), None => Err("n2") } } None => Err("n1") }
}
module check {
  pub fn try_val(v: Int, min: Int) -> Option[Int] { return validate(v, min); }
  pub fn merge(a: Option[Int], b: Option[Int]) -> Result[Int, Str] { return combine(a, b); }
}
use check.try_val;
use check.merge;
fn main() -> Int {
  var arr = [3, 7, 2, 9, 1];
  var i = 0;
  var sum = 0;
  while i < 5 { sum += arr[i]; i += 1; }
  var o1 = try_val(sum, 0);
  var o2 = try_val(10, 0);
  match merge(o1, o2) {
    Ok(v) => { if v == 32 { return 0; } }
    Err(_) => { return 1; }
  }
  return 2;
}
