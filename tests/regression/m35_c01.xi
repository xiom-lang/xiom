// M35-C01: if without else — single branch, no alternative path
fn main() -> Int {
  var x: Int = 0;
  if x == 0 { x = 42; }
  if x != 42 { return 1; }
  return 0;
}
