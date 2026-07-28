fn div2(a: Int) -> Result[Int, Str] {
  if a % 2 == 0 { return Ok(a / 2); }
  return Err("odd");
}
fn bind_res(r: Result[Int, Str], f: fn(Int) -> Result[Int, Str]) -> Result[Int, Str] {
  match r { Ok(v) => f(v), Err(e) => Err(e) }
}
fn main() -> Int {
  var r1 = bind_res(bind_res(Ok(40), div2), div2);
  if r1.is_ok() { var v = r1.unwrap(); if v != 10 { return 1; } } else { return 2; }
  var r2 = bind_res(Ok(5), div2);
  if !r2.is_err() { return 3; }
  var r3 = bind_res(Err("fail"), div2);
  if !r3.is_err() { return 4; }
  return 0;
}
