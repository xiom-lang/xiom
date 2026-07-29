// M32: Bool false as Int should be 0
fn main() -> Int {
  var b: Bool = false;
  var n: Int = b as Int;
  if n == 0 { return 0; }
  return 1;
}
