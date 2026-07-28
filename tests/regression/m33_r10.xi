fn consume_opt(o: Option[Int]) -> Int {
  match o { Some(v) => v * 2, None => -1 }
}
fn make_opt(v: Int) -> Option[Int] {
  if v >= 0 { return Some(v); }
  return None;
}
fn main() -> Int {
  if consume_opt(Some(7)) != 14 { return 1; }
  if consume_opt(None) != -1 { return 2; }
  if consume_opt(make_opt(3)) != 6 { return 3; }
  if consume_opt(make_opt(-1)) != -1 { return 4; }
  return 0;
}
