// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z26: invariant+contract_chain+generic+enum+struct+module+match+while+compound_assign+derive+Option
type SafeInt = { val: Int; invariant: val >= 0; } derive[Eq]
enum SafeOp { Add(n: Int), Sub(n: Int), Mul(n: Int), Sqrt }
fn SafeInt.new(v: Int) -> SafeInt
  ensures: result.val >= 0
{
  if v < 0 { return SafeInt{ val: 0; }; }
  return SafeInt{ val: v; };
}
fn SafeInt.saturating_add(self, n: Int) -> SafeInt
  ensures: result.val >= 0
{ var r = self; r.val += n; if r.val < 0 { r.val = 0; } return r; }
fn safe_op[T](s: SafeInt, op: SafeOp) -> SafeInt
  requires: s.val >= 0
  ensures: result.val >= 0
{
  match op {
    Add(n) => s.saturating_add(n),
    Sub(n) => s.saturating_add(-n),
    Mul(n) => { var r = s; r.val *= n; if r.val < 0 { r.val = 0; } return r; }
    Sqrt => { var r = s; var v = r.val / 2; return SafeInt{ val: v; }; }
  }
}
module safe_math {
  pub fn op(s: SafeInt, o: SafeOp) -> SafeInt { return safe_op(s, o); }
  pub fn from_int(v: Int) -> SafeInt { return SafeInt.new(v); }
  pub fn into_option(s: SafeInt) -> Option[Int] { if s.val > 0 { return Some(s.val); } return None; }
}
use safe_math.op;
use safe_math.from_int;
use safe_math.into_option;
fn main() -> Int {
  var s1 = from_int(10);
  var s2 = op(s1, SafeOp.Add(5));
  var s3 = op(s2, SafeOp.Sub(7));
  var s4 = op(s3, SafeOp.Mul(2));
  var chk = 0;
  if s2.val == 15 { chk += 1; }
  if s3.val == 8 { chk += 1; }
  if s4.val == 16 { chk += 1; }
  match into_option(s4) { Some(v) => { if v == 16 { chk += 1; } } None => {} }
  if chk == 4 { return 0; }
  return 1;
}
