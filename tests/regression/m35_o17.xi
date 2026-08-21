// M35-O17: Option transpose -- simulate Option[Result] pattern using flat matches
fn div_result(a: Int, b: Int) -> Result[Int, Str] {
  if b == 0 { return Err("div0"); }
  return Ok(a / b);
}
fn main() -> Int {
  match div_result(10, 2) { Ok(v) => { if v != 5 { return 1; } } Err(_) => { return 2; } }
  match div_result(8, 0) { Ok(_) => { return 3; } Err(_) => {} }
  var opt: Option[Int] = Some(0);
  match opt { Some(v) => { if v != 0 { return 4; } } None => { return 5; } }
  var opt2: Option[Int] = None;
  match opt2 { Some(_) => { return 6; } None => {} }
  return 0;
}
