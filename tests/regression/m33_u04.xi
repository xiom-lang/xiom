// M33-U04: Null pointer — create null pointer and compare
fn main() -> Int {
  var p: *Int;
  unsafe { p = 0 as *Int; }
  if unsafe { p == (0 as *Int) } { return 0; }
  return 1;
}
