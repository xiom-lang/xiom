// M34-Q08: Contract with Result -- requires/ensures involving Result types
fn safe_div(a: Int, b: Int) -> Result[Int, Str]
  requires: b != 0
{
  return Ok(a / b);
}
fn div_chain(a: Int, b: Int, c: Int) -> Result[Int, Str]
  requires: b != 0
  ensures: result.is_ok || result.is_err
{
  var r = safe_div(a, b);
  match r { Ok(v) => safe_div(v, c), Err(e) => Err(e) }
}
fn unwrap_or_default(r: Result[Int, Str], d: Int) -> Int
  ensures: result == d || result != d
{
  match r { Ok(v) => v, Err(_) => d }
}
fn main() -> Int {
  var r1 = div_chain(100, 5, 4);
  var v1 = unwrap_or_default(r1, 0);
  var r2 = safe_div(33, 11);
  var v2 = unwrap_or_default(r2, 0);
  if v1 == 5 && v2 == 3 { return 0; }
  return 1;
}
