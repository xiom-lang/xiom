// M32: Struct field mutation with narrow ints
type Counter = { val: UInt16 };
fn main() -> Int {
  var c: Counter = Counter{ val: 65535 };
  c.val = c.val + 1 as UInt16;
  if c.val == 0 as UInt16 { return 0; }
  return 1;
}
