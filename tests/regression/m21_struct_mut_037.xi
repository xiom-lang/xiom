module m21_struct_mut_037
fn main() -> Int {
  var b: Box[Int] = { val: 10; };
  b.val = 42;
  if b.val == 42 { return 0; }
  return 1;
  return 1;
}
