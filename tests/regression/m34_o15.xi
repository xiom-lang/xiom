// M34-O15: Option chain with match propagation — sequential Option unwraps (fixed)
fn try_half(a: Int) -> Option[Int] {
  if a % 2 == 0 { return Some(a / 2); }
  return None;
}
fn try_square(a: Int) -> Option[Int] {
  if a >= 0 && a < 1000 { return Some(a * a); }
  return None;
}
fn chain_opt(a: Int) -> Option[Int] {
  match try_half(a) {
    Some(v) => try_square(v),
    None => None
  }
}
fn main() -> Int {
  match chain_opt(16) { Some(v) => { if v != 64 { return 1; } } None => { return 2; } }
  match chain_opt(3) { Some(_) => { return 3; } None => {} }
  match chain_opt(0) { Some(v) => { if v != 0 { return 4; } } None => { return 5; } }
  match chain_opt(-10) { Some(_) => { return 6; } None => {} }
  return 0;
}
