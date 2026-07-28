// M33-U05: Pointer to Int — read value through pointer deref
fn main() -> Int {
  var x: Int = 7;
  var p: *Int;
  unsafe { p = &x as *Int; }
  var r: Int;
  unsafe { r = *p; }
  if r == 7 { return 0; }
  return 1;
}
