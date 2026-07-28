// M34-O14: ? with custom error type — ? on Result[T, MyError]
enum CalcError { Overflow, Underflow, DivByZero }
fn mul(a: Int, b: Int) -> Result[Int, CalcError] {
  if a > 10000 || b > 10000 { return Err(CalcError.Overflow); }
  return Ok(a * b);
}
fn div(a: Int, b: Int) -> Result[Int, CalcError] {
  if b == 0 { return Err(CalcError.DivByZero); }
  return Ok(a / b);
}
fn eval(a: Int, b: Int, c: Int) -> Result[Int, CalcError] {
  var m = mul(a, b)?;
  var d = div(m, c)?;
  return Ok(d);
}
fn main() -> Int {
  match eval(10, 5, 2) { Ok(v) => { if v != 25 { return 1; } } Err(_) => { return 2; } }
  match eval(10, 5, 0) { Ok(_) => { return 3; } Err(_) => {} }
  match eval(10, 5, 2) { Ok(v) => { if v != 25 { return 4; } } Err(_) => { return 5; } }
  return 0;
}
