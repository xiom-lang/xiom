// M34-Q07: Contract with Option — requires/ensures involving Option types
fn get_positive(x: Int) -> Option[Int]
  requires: x > -100
  ensures: result.is_some || result.is_none
{
  if x > 0 { return Some(x); }
  return None;
}
fn unwrap_or_zero(o: Option[Int]) -> Int
  ensures: result >= 0
{
  match o { Some(v) => v, None => 0 }
}
fn chain_option(x: Int) -> Int
  requires: x >= 0
  ensures: result >= 0
{
  var o = get_positive(x);
  return unwrap_or_zero(o);
}
fn main() -> Int {
  var r1 = chain_option(10);
  var r2 = chain_option(0);
  var r3 = chain_option(5);
  if r1 == 10 && r2 == 0 && r3 == 5 { return 0; }
  return 1;
}
