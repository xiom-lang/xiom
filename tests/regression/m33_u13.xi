// M33-U13: Pointer return — function returns a pointer
fn get_addr() -> *Int {
  var x: Int = 42;
  unsafe { return &x as *Int; }
}
fn main() -> Int {
  var p: *Int = get_addr();
  if unsafe { *p } == 42 { return 0; }
  return 1;
}
