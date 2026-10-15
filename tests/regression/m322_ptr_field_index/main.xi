// m322 (release gate): a FIXED struct field holding a scalar pointer must
// index by ELEMENT width with a RUNTIME index -- pre-fix `b.data[i]` lowered
// to a byte-stride i8 GEP + i8 load (only the i=0 byte pattern happened to
// look right), so sums over the buffer read one byte per element.
module m322_ptr_field_index

type Buf = { data: *Int; len: Int; }

fn at(buf: &Buf, i: Int) -> Int {
  return buf.data[i];
}

fn sum(buf: &Buf) -> Int {
  var s: Int = 0;
  var i: Int = 0;
  while i < buf.len { s = s + buf.data[i]; i = i + 1; }
  return s;
}

fn main() -> Int {
  var arr = [10, 20, 30];
  var b = Buf{ data: &arr[0]; len: 3; };
  if at(&b, 2) != 30 { return 1; }
  if sum(&b) != 60 { return 2; }
  if at(&b, 0) != 10 { return 3; }
  return 0;
}
