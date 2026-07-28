// M35-O12: Option map_or — match-based map_or (map then unwrap_or)
fn opt_map_or(o: Option[Int], default: Int, f: fn(Int) -> Int) -> Int {
  match o { Some(v) => f(v), None => default }
}
fn triple(x: Int) -> Int { return x * 3; }
fn main() -> Int {
  if opt_map_or(Some(5), 0, triple) != 15 { return 1; }
  if opt_map_or(None, 99, triple) != 99 { return 2; }
  if opt_map_or(Some(0), 100, triple) != 0 { return 3; }
  if opt_map_or(Some(-4), 10, triple) != -12 { return 4; }
  return 0;
}
