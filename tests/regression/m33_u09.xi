// M33-U09: Unsafe read from pointer — deref and use value
fn main() -> Int {
  var v: Int = 123;
  var p: *Int;
  unsafe { p = &v as *Int; }
  var read: Int;
  unsafe { read = *p; }
  if read == 123 { return 0; }
  return 1;
}
