// M33-U15: Extern function returning pointer -- declare only, no call
extern "C" {
  fn get_magic() -> *Int;
}
fn main() -> Int {
  var p: *Int;
  unsafe { p = 0 as *Int; }
  if unsafe { p == (0 as *Int) } { return 0; }
  return 1;
}
