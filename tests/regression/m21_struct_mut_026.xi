module m21_struct_mut_026
type Pair = { first: Int; second: Int; }
fn main() -> Int {
  var p: Pair = Pair{ first: 10; second: 20; };
  var p2 = swap(p);
  if p2.first == 20 && p2.second == 10 { return 0; }
  return 1;
  return 1;
}
