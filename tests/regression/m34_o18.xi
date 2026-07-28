// M34-O18: ? with contract — ? inside contract-guarded function (fixed)
fn safe_sqrt(n: Int) -> Result[Int, Int]
  requires: n >= 0
{
  if n == 0 { return Ok(0); }
  if n == 1 { return Ok(1); }
  if n == 4 { return Ok(2); }
  if n == 9 { return Ok(3); }
  if n == 16 { return Ok(4); }
  if n == 25 { return Ok(5); }
  if n == 36 { return Ok(6); }
  return Err(-1);
}
fn chain_mult(n: Int) -> Result[Int, Int]
  requires: n >= 0
{
  var s1 = safe_sqrt(n)?;
  var s2 = safe_sqrt(n * 4)?;
  return Ok(s1 + s2);
}
fn main() -> Int {
  match chain_mult(4) { Ok(v) => { if v != 6 { return 1; } } Err(_) => { return 2; } }
  match chain_mult(1) { Ok(v) => { if v != 3 { return 3; } } Err(_) => { return 4; } }
  match chain_mult(0) { Ok(v) => { if v != 0 { return 5; } } Err(_) => { return 6; } }
  return 0;
}
