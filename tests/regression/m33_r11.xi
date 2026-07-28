fn parse(s: Str) -> Result[Int, Str] {
  if s == "" { return Err("empty"); }
  if s == "good" { return Ok(1); }
  return Err("unknown");
}
fn main() -> Int {
  match parse("good") { Ok(v) => { if v != 1 { return 1; } } Err(_) => { return 2; } }
  match parse("") { Ok(_) => { return 3; } Err(e) => { if e != "empty" { return 4; } } }
  match parse("bad") { Ok(_) => { return 5; } Err(_) => {} }
  return 0;
}
