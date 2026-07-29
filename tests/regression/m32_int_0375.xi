// M32: elif chain with UInt8 boundary values
fn main() -> Int {
  var v: UInt8 = 255;
  if v < 128 as UInt8 { return 1; }
  elif v < 255 as UInt8 { return 1; }
  elif v == 255 as UInt8 { return 0; }
  else { return 1; }
}
