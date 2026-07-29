// M32: Nested struct with multiple narrow int types
type Inner = { a: Int8; b: UInt16 };
fn main() -> Int {
  type Outer = { inner: Inner; tag: UInt8 };
  var o: Outer = Outer{ inner: Inner{ a: -1 as Int8, b: 65535 }; tag: 0 };
  var casted: UInt8 = o.inner.a as UInt8;
  if casted == 255 as UInt8 { return 0; }
  return 1;
}
