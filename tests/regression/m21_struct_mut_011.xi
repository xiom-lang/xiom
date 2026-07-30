module m21_struct_mut_011
type Counter = { value: Int; }
fn main() -> Int {
  var c: Counter = Counter{ value: 0; };
  c.value = c.value + 1;
  c.value = c.value + 1;
  if c.value == 2 { return 0; }
  return 1;
}
