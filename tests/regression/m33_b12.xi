// M33-B12: Borrow in while scope -- &mut reference used inside loop, conditionally
fn dec(x: &mut Int) { *x = *x - 1; }
fn main() -> Int {
  var a = 5;
  var count = 0;
  while a > 0 {
    dec(&mut a);
    count = count + 1;
  }
  if a == 0 && count == 5 { return 0; }
  return 1;
}
