// M22: Struct field mutation and nested access
type Inner = { val: Int; }
type Outer = { name: Str; inner: Inner; }
fn main() -> Int {
  var i = Inner{ val: 10; };
  var o = Outer{ name: "test"; inner: i; };
  o.inner.val = 99;
  return o.inner.val;
}
