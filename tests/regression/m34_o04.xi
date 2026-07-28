// M34-O04: 3-chain ? — three consecutive ? in sequence
fn validate(v: Int) -> Result[Int, Str] {
  if v < 0 { return Err("negative"); }
  return Ok(v);
}
fn triple(a: Int) -> Result[Int, Str] {
  var x = validate(a)?;
  var y = validate(x + 10)?;
  var z = validate(y + 100)?;
  return Ok(z);
}
fn main() -> Int {
  match triple(0) { Ok(v) => { if v != 110 { return 1; } } Err(_) => { return 2; } }
  match triple(-1) { Ok(_) => { return 3; } Err(_) => {} }
  match triple(-5) { Ok(_) => { return 4; } Err(_) => {} }
  return 0;
}
