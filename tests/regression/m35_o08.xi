// M35-O08: Option map -- match-based map implementation
fn option_map_int(o: Option[Int], f: fn(Int) -> Int) -> Option[Int] {
  match o { Some(v) => Some(f(v)), None => None }
}
fn double(x: Int) -> Int { return x * 2; }
fn square(x: Int) -> Int { return x * x; }
fn main() -> Int {
  var a = Some(5);
  var b: Option[Int] = None;
  match option_map_int(a, double) { Some(v) => { if v != 10 { return 1; } } None => { return 2; } }
  match option_map_int(b, double) { Some(_) => { return 3; } None => {} }
  match option_map_int(Some(7), square) { Some(v) => { if v != 49 { return 4; } } None => { return 5; } }
  match option_map_int(Some(0), double) { Some(v) => { if v != 0 { return 6; } } None => { return 7; } }
  return 0;
}
