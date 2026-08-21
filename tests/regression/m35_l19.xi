// M35-L19: Pointer to array -- pointer to separate vars simulating array
fn main() -> Int {
  var a: Int = 11;
  var e: Int = 55;
  var pa: *Int;
  var pe: *Int;
  unsafe { pa = &a as *Int; }
  unsafe { pe = &e as *Int; }
  var va: Int;
  var ve: Int;
  unsafe { va = *pa; }
  unsafe { ve = *pe; }
  if va != 11 { return 1; }
  if ve != 55 { return 2; }
  return 0;
}
