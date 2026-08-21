// M35-C24: match with guard -- pattern matching with if condition inside arm body
enum Value { Num(n: Int), Fail(msg: Str), Nothing }
fn process(v: Value) -> Int {
  match v {
    Num(n) => { if n > 0 { return n; } else { return 0; } }
    Fail(_) => { return -1; }
    Nothing => { return -2; }
  }
}
fn safe_sqrt(v: Value) -> Int {
  match v {
    Num(n) => { if n >= 0 { return n * n; } else { return 0; } }
    _ => { return -1; }
  }
}
fn main() -> Int {
  if process(Value.Num(10)) != 10 { return 1; }
  if process(Value.Num(-5)) != 0 { return 2; }
  if process(Value.Fail("fail")) != -1 { return 3; }
  if process(Value.Nothing) != -2 { return 4; }
  if safe_sqrt(Value.Num(4)) != 16 { return 5; }
  if safe_sqrt(Value.Num(-4)) != 0 { return 6; }
  if safe_sqrt(Value.Fail("x")) != -1 { return 7; }
  return 0;
}
