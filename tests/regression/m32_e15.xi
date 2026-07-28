// M32-E15: Match with expression arms and wildcard in nested position
enum Opt { None, Some(val: Int) }
fn unwrap_or(o: Opt, default: Int) -> Int {
  match o {
    None => default,
    Some(v) => v,
  }
}
fn main() -> Int {
  if unwrap_or(Opt.None, 42) != 42 { return 1; }
  if unwrap_or(Opt.Some(7), 0) != 7 { return 2; }
  return 0;
}
