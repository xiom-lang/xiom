// M32: Bool as Int (should be 0 or 1, not arbitrary values)
fn main() -> Int {
  var b: Bool = true;
  var n: Int = b as Int;
  if n == 1 { return 0; }
  return 1;
}
