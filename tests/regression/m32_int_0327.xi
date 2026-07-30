// M32: Bool-casting with false
fn main() -> Int {
  var b: Bool = false;
  var as_int: Int = if b { 1 } else { 0 };
  if as_int == 0 { return 0; }
  return 1;
}
