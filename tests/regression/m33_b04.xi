// M33-B04: Exclusive write borrow — single &mut with exclusive access
fn add_five(x: &mut Int) { *x = *x + 5; }
fn main() -> Int {
  var a = 20;
  add_five(&mut a);
  add_five(&mut a);
  if a == 30 { return 0; }
  return 1;
}
