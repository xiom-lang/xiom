// M35-O22: Result and_then -- match-based and_then chaining
fn safe_div(a: Int, b: Int) -> Result[Int, Str] {
  if b == 0 { return Err("div0"); }
  return Ok(a / b);
}
fn div_next(x: Int) -> Result[Int, Str] { return safe_div(100, x); }
fn res_and_then(r: Result[Int, Str], f: fn(Int) -> Result[Int, Str]) -> Result[Int, Str] {
  match r { Ok(v) => f(v), Err(e) => Err(e) }
}
fn main() -> Int {
  var r = safe_div(10, 2);
  match r { Ok(v) => { if v != 5 { return 1; } } Err(_) => { return 2; } }
  match res_and_then(r, div_next) { Ok(v) => { if v != 20 { return 3; } } Err(_) => { return 4; } }
  match res_and_then(safe_div(10, 0), div_next) { Ok(_) => { return 5; } Err(e) => { if e != "div0" { return 6; } } }
  return 0;
}
