// M33-B10: Var reassignment -- rebind var, verify old value replaced
fn main() -> Int {
  var x = 5;
  x = 10;
  x = x + 5;
  if x == 15 { return 0; }
  return 1;
}
