module m21_struct_mut_032
type Pair = { a: Int; b: Int; }
fn main() -> Int {
  var p: Pair = Pair{ a: 5; b: 9; };
  var swapped = swap_fields(p);
  if swapped.a == 9 && swapped.b == 5 { return 0; }
  return 1;
  return 1;
}
