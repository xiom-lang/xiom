// M32: Result UInt8 Ok with high value
fn main() -> Int {
  var r: Result[UInt8, Bool] = Ok(250);
  var v: UInt8 = r.unwrap();
  if v == 250 as UInt8 { return 0; }
  return 1;
}
