// M33-U11: Multiple extern function declarations -- two extern blocks
extern "C" {
  fn c_add(a: Int, b: Int) -> Int;
}
extern "C" {
  fn c_mul(a: Int, b: Int) -> Int;
}
fn main() -> Int {
  var x: Int = 5;
  var p: *Int;
  unsafe { p = &x as *Int; }
  if unsafe { *p } == 5 { return 0; }
  return 1;
}
