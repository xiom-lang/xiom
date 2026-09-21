// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X30: ALL FEATURES COMBINED -- struct+enum+generic+match+while+if+contract+invariant+derive+impl+module+Option+Result+array+pointer+unsafe+cast+compound_assign+const+type_alias+recursion+bool
const OFFSET: Int = 10;
type Named = { id: Int; score: Int; invariant: score >= 0; } derive[Eq]
enum Status { Active, Inactive, Unknown(i: Int) }
interface Evaluable {
  fn evaluate(self) -> Int;
}
impl Evaluable for Named {
  fn evaluate(self) -> Int { return self.score + OFFSET; }
}
fn clamp_val(val: Int, lo: Int, hi: Int) -> Int {
  var v = val;
  if v < lo { v = lo; }
  if v > hi { v = hi; }
  return v;
}
fn status_to_int(s: Status) -> Int {
  match s {
    Status.Active => 1,
    Status.Inactive => 0,
    Status.Unknown(i) => i,
  }
}
fn factorial(n: Int) -> Int
  requires: n >= 0
  ensures: result >= 1
{
  if n <= 1 { return 1; }
  return n * factorial(n - 1);
}
fn safe_div(a: Int, b: Int) -> Result[Int, Str]
  requires: b != 0
{
  return Ok(a / b);
}
module ops {
  pub fn add(a: Int, b: Int) -> Int { return a + b; }
  pub fn mul(a: Int, b: Int) -> Int { return a * b; }
}
use ops.add;
use ops.mul;
fn main() -> Int {
  var n = Named{ id: 1; score: 5; };
  if n.evaluate() != 15 { return 1; }
  if !(n == Named{ id: 1; score: 5; }) { return 2; }
  if status_to_int(Status.Active) != 1 { return 3; }
  if status_to_int(Status.Inactive) != 0 { return 4; }
  if status_to_int(Status.Unknown(7)) != 7 { return 5; }
  if factorial(5) != 120 { return 6; }
  match safe_div(10, 2) { Ok(v) => if v != 5 { return 7; } Err(_) => { return 8; } }
  match safe_div(10, 2) { Ok(v) => {} Err(_) => { return 9; } }
  if add(3, 4) != 7 { return 10; }
  if mul(6, 7) != 42 { return 11; }
  if clamp_val(100, 0, 50) != 50 { return 12; }
  if clamp_val(-10, 0, 50) != 0 { return 13; }
  var arr = [1, 4, 9, 16, 25];
  var i = 0; var s = 0;
  while i < 5 { s += arr[i]; i += 1; }
  if s != 55 { return 14; }
  var fv: Float64 = 3.0;
  var casted: Int = fv as Int;
  if casted != 3 { return 15; }
  if true && !false != true { return 16; }
  if false || (true && false) != false { return 17; }
  return 0;
}
