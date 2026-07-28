// M35-O27: Result[Bool,Bool] — bool success, bool error
fn validate(x: Int) -> Result[Bool, Bool] {
  if x > 0 { return Ok(true); }
  return Err(false);
}
fn main() -> Int {
  match validate(5) { Ok(v) => { if v != true { return 1; } } Err(_) => { return 2; } }
  match validate(0) { Ok(_) => { return 3; } Err(e) => { if e != false { return 4; } } }
  match validate(-1) { Ok(_) => { return 5; } Err(e) => { if e != false { return 6; } } }
  return 0;
}
