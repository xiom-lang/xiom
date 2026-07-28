fn opt_map(o: Option[Int], f: fn(Int) -> Int) -> Option[Int] {
  match o { Some(v) => Some(f(v)), None => None }
}
fn double(x: Int) -> Int { return x * 3; }
fn main() -> Int {
  match opt_map(Some(5), double) { Some(v) => { if v != 15 { return 1; } } None => { return 2; } }
  match opt_map(None, double) { Some(_) => { return 3; } None => {} }
  return 0;
}
