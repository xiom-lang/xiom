// M33-U03: Pointer creation -- declare pointer var and assign via cast
fn main() -> Int {
  var a: Int = 10;
  var p: *Int;
  unsafe { p = &a as *Int; }
  if unsafe { *p } == 10 { return 0; }
  return 1;
}
