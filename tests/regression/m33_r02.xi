fn div(a: Int, b: Int) -> Result[Int, Str] { if b == 0 { return Err("div0"); } return Ok(a / b); }
fn main() -> Int {
  match div(10, 2) { Ok(v) => { if v != 5 { return 1; } } Err(_) => { return 2; } }
  match div(10, 0) { Ok(_) => { return 3; } Err(_) => {} }
  if div(15, 3).is_ok() && div(15, 0).is_err() { return 0; }
  return 4;
}
