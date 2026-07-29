// M32: Narrow int in while loop counter
fn main() -> Int {
  var i: UInt8 = 0;
  while i < 5 as UInt8 {
    i = i + 1 as UInt8;
  }
  if i == 5 as UInt8 { return 0; }
  return 1;
}
