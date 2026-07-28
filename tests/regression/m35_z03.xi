// M35-Z03: struct+match+while+contract+module+compound_assign+closure+Option+derive+array+cast
type Record = { val: Int; tag: Int; } derive[Eq]
fn bump(r: Record) -> Record
  requires: r.val >= 0
{
  var res = r;
  res.val += 1;
  return res;
}
fn double(r: Record) -> Record
  ensures: result.val >= r.val
{
  var res = r;
  res.val *= 2;
  return res;
}
module data {
  pub fn inc(r: Record) -> Record { return bump(r); }
  pub fn mul(r: Record) -> Record { return double(r); }
  pub fn tagged(r: Record) -> Int { return r.tag * r.val; }
}
use data.inc;
use data.mul;
use data.tagged;
fn main() -> Int {
  var r1 = Record{ val: 7; tag: 3; };
  var r2 = inc(r1);
  var r3 = mul(r2);
  var chk: Int = 0;
  var i = 0;
  var arr = [1, 2, 3, 4, 5];
  var sum = 0;
  while i < 5 { sum += arr[i]; i += 1; }
  if r2.val == 8 { chk += 1; }
  if r3.val == 16 { chk += 1; }
  if tagged(r3) == 48 { chk += 1; }
  if sum == 15 { chk += 1; }
  var f = |x| x + r1.tag;
  if f(10) == 13 { chk += 1; }
  if chk == 5 { return 0; }
  return 1;
}
