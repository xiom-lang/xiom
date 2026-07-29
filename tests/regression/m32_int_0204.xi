// M32: Option UInt8 payload
fn main() -> Int {
  var opt: Option[UInt8] = Some(255);
  if opt.unwrap() == 255 as UInt8 { return 0; }
  return 1;
}
