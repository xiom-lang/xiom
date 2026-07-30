module m21_struct_mut_014
type Num = { val: Int; }
fn main() -> Int {
  var n: Num = Num{ val: 5; };
  n.val = n.val + 3;
  n.val = n.val * 2;
  if n.val == 16 { return 0; }
  return 1;
}
