// M35-T26: Struct exhaustive -- nested structs
type Inner = { val: Int; }
type Middle = { inner: Inner; flag: Bool; }
type Outer = { mid: Middle; label: Str; count: Float64; }
type AllFields = { b: Bool; i: Int; f: Float64; s: Str; }
fn build_outer() -> Outer {
  return Outer{ mid: Middle{ inner: Inner{ val: 42 }; flag: true; }; label: "outer"; count: 3.0; };
}
fn outer_access(o: Outer) -> Int {
  var s: Int = 0;
  s = s + o.mid.inner.val;
  if o.mid.flag { s = s + 1; }
  return s;
}
fn all_fields_sum(af: AllFields) -> Int {
  var s: Int = 0;
  if af.b { s = s + 1; }
  s = s + af.i;
  s = s + af.f as Int;
  s = s + af.s.len() as Int;
  return s;
}
fn main() -> Int {
  var o: Outer = build_outer();
  if o.mid.inner.val != 42 { return 1; }
  var s: Int = outer_access(o);
  if s != 43 { return 2; }
  var af: AllFields = AllFields{ b: true; i: 10; f: 3.0; s: "hi"; };
  var afs: Int = all_fields_sum(af);
  var expected: Int = 1 + 10 + 3 + 2;
  if afs != expected { return 3; }
  return 0;
}

