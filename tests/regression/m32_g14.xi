// M32-G14: Generic enum matching with Option and Result
fn unwrap_or(o: Option[Int], default: Int) -> Int {
  match o {
    Some(v) => { return v; }
    None => { return default; }
  }
}
fn main() -> Int {
  var a: Option[Int] = Some(42);
  var b: Option[Int] = None;
  var r1 = unwrap_or(a, 0);
  var r2 = unwrap_or(b, 99);
  if r1 != 42 { return 1; }
  if r2 != 99 { return 2; }
  return 0;
}
