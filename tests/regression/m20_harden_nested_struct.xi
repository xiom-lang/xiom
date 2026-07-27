type Inner = { value: Int; }
type Middle = { inner: Inner; }
type Outer = { middle: Middle; }
fn main() -> Int {
  var o = Outer{ middle: Middle{ inner: Inner{ value: 42 } } };
  if o.middle.inner.value != 42 { return 1; }
  o.middle.inner.value = 99;
  if o.middle.inner.value != 99 { return 2; }
  return 0;
}