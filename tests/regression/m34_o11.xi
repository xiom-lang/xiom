// M34-O11: ? with early return — ? followed by early return in branch
fn check(a: Int) -> Result[Int, Str] {
  if a < 0 { return Err("negative"); }
  return Ok(a);
}
fn early(a: Int, b: Int) -> Result[Int, Str] {
  var x = check(a)?;
  if x > 100 { return Err("too big"); }
  var y = check(b)?;
  if y > 100 { return Err("too big"); }
  return Ok(x + y);
}
fn main() -> Int {
  match early(10, 20) { Ok(v) => { if v != 30 { return 1; } } Err(_) => { return 2; } }
  match early(-1, 20) { Ok(_) => { return 3; } Err(_) => {} }
  match early(200, 10) { Ok(_) => { return 4; } Err(_) => {} }
  return 0;
}
