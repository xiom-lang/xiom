// M32: Option UInt16 with high value
fn main() -> Int {
  var opt: Option[UInt16] = Some(60000);
  var v: UInt16 = opt.unwrap();
  if v == 60000 as UInt16 { return 0; }
  return 1;
}
