// M34-O16: Result chain with map+? -- Result.map combined with ?
fn parse_int(s: Str) -> Result[Int, Str] {
  if s == "0" { return Ok(0); }
  if s == "1" { return Ok(1); }
  if s == "42" { return Ok(42); }
  return Err("not a number");
}
fn double_if_possible(s: Str) -> Result[Int, Str] {
  var n = parse_int(s)?;
  return Ok(n * 2);
}
fn compute_chain(s1: Str, s2: Str) -> Result[Int, Str] {
  var a = double_if_possible(s1)?;
  var b = double_if_possible(s2)?;
  return Ok(a + b);
}
fn main() -> Int {
  match compute_chain("1", "42") { Ok(v) => { if v != 86 { return 1; } } Err(_) => { return 2; } }
  match compute_chain("0", "1") { Ok(v) => { if v != 2 { return 3; } } Err(_) => { return 4; } }
  match compute_chain("1", "bad") { Ok(_) => { return 5; } Err(_) => {} }
  return 0;
}
