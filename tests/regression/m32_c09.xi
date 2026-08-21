// M32-C09: Type invariant -- Positive integer struct
type Positive = { val: Int; invariant: val > 0; }
fn main() -> Int {
  var p = Positive{ val: 100; };
  if p.val > 0 { return 0; }
  return 1;
}
