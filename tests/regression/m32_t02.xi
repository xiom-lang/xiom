// M32-T02: Char literal — declare and cast to Int
fn main() -> Int {
  var c: Char = 'A';
  if c as Int == 65 { return 0; }
  return 1;
}
