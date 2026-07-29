// M32: Option Int16 payload
fn main() -> Int {
  var opt: Option[Int16] = Some(32767);
  if opt.is_some() { return 0; }
  return 1;
}
