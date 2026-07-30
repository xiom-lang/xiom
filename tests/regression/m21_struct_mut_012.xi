module m21_struct_mut_012
type Counter = { value: Int; }
fn main() -> Int {
  var c: Counter = Counter{ value: 100; };
  c.value = 0;
  if c.value == 0 { return 0; }
  return 1;
}
