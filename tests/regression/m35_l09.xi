// M35-L09: Struct with nested struct — verify nested struct layout
type Inner = { x: Int; y: Int; }
type Outer = { id: Int; inner: Inner; flag: Bool; }

fn main() -> Int {
  var inn = Inner{ x: 5; y: 10; };
  var out = Outer{ id: 1; inner: inn; flag: true; };
  if out.id != 1 { return 1; }
  if out.inner.x != 5 { return 2; }
  if out.inner.y != 10 { return 3; }
  if out.flag != true { return 4; }
  return 0;
}
