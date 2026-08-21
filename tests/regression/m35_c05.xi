// M35-C05: nested if 5 deep -- cascading conditional logic
fn deep_nest(a: Int, b: Int, c: Int, d: Int, e: Int) -> Int {
  if a > 0 {
    if b > 0 {
      if c > 0 {
        if d > 0 {
          if e > 0 { return 1; }
          else { return 2; }
        }
        else { return 3; }
      }
      else { return 4; }
    }
    else { return 5; }
  }
  return 6;
}
fn main() -> Int {
  if deep_nest(1, 1, 1, 1, 1) != 1 { return 1; }
  if deep_nest(1, 1, 1, 1, 0) != 2 { return 2; }
  if deep_nest(1, 1, 1, 0, 5) != 3 { return 3; }
  if deep_nest(1, 1, 0, 5, 5) != 4 { return 4; }
  if deep_nest(1, 0, 5, 5, 5) != 5 { return 5; }
  if deep_nest(0, 5, 5, 5, 5) != 6 { return 6; }
  return 0;
}
