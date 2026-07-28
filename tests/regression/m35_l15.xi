// M35-L15: Pointer difference — compare two related pointers
fn main() -> Int {
  var x: Int = 5;
  var y: Int = 10;
  var px: *Int;
  var py: *Int;
  unsafe { px = &x as *Int; }
  unsafe { py = &y as *Int; }
  if unsafe { px == py } { return 1; }
  if unsafe { px != py } {
    if unsafe { *px } == 5 && unsafe { *py } == 10 { return 0; }
  }
  return 2;
}
