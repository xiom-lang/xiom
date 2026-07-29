// M32: Option Int8 payload
fn main() -> Int {
  var opt: Option[Int8] = Some(-128 as Int8);
  if opt == Some(-128 as Int8) { return 0; }
  return 1;
}
