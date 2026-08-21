// M34-O06: ? in function returning Result -- ? inside nested computation
fn safe_mul(a: Int, b: Int) -> Result[Int, Str] {
  if a > 1000 { return Err("overflow"); }
  if b > 1000 { return Err("overflow"); }
  return Ok(a * b);
}
fn divide(a: Int, b: Int) -> Result[Int, Str] {
  if b == 0 { return Err("div0"); }
  return Ok(a / b);
}
fn compute(a: Int, b: Int, c: Int) -> Result[Int, Str] {
  var mul = safe_mul(a, b)?;
  var div = divide(mul, c)?;
  return Ok(div);
}
fn main() -> Int {
  match compute(10, 5, 2) { Ok(v) => { if v != 25 { return 1; } } Err(_) => { return 2; } }
  match compute(10, 5, 0) { Ok(_) => { return 3; } Err(_) => {} }
  match compute(2000, 1, 1) { Ok(_) => { return 4; } Err(_) => {} }
  return 0;
}
