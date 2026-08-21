// M33-U07: Pointer arithmetic -- use two pointers to adjacent vars
fn main() -> Int {
  var a: Int = 10;
  var b: Int = 20;
  var pa: *Int;
  var pb: *Int;
  unsafe { pa = &a as *Int; }
  unsafe { pb = &b as *Int; }
  if unsafe { *pa } == 10 && unsafe { *pb } == 20 { return 0; }
  return 1;
}
