// M33-U16: Mixed safe/unsafe -- deref to temp, then safe arithmetic
fn main() -> Int {
  var a: Int = 10;
  var b: Int = 20;
  var pa: *Int;
  var pb: *Int;
  unsafe { pa = &a as *Int; }
  unsafe { pb = &b as *Int; }
  var va: Int;
  var vb: Int;
  unsafe { va = *pa; }
  unsafe { vb = *pb; }
  var sum: Int = va + vb;
  if sum == 30 { return 0; }
  return 1;
}
