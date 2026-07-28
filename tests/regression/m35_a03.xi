// M35-A03: Verify sortedness — check if arrays are in ascending order
fn main() -> Int {
  var a = [1, 2, 3, 4, 5, 6, 7];
  var c = [7, 6, 5, 4, 3, 2, 1];
  var na: Int = 7;
  var sorted_a: Int = 1;
  var i: Int = 1;
  while i < na { if a[i - 1] > a[i] { sorted_a = 0; } i = i + 1; }
  if sorted_a != 1 { return 1; }
  var nc: Int = 7;
  var sorted_c: Int = 1;
  i = 1;
  while i < nc { if c[i - 1] > c[i] { sorted_c = 0; } i = i + 1; }
  if sorted_c != 0 { return 2; }
  return 0;
}
