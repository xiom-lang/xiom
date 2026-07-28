// M35-L14: Pointer arithmetic — deref and sum integer values from multiple pointers
fn main() -> Int {
  var a: Int = 10;
  var b: Int = 20;
  var c: Int = 30;
  var d: Int = 40;
  var va: Int;
  var vb: Int;
  var vc: Int;
  var vd: Int;
  var pa: *Int;
  var pb: *Int;
  var pc: *Int;
  var pd: *Int;
  unsafe { pa = &a as *Int; }
  unsafe { pb = &b as *Int; }
  unsafe { pc = &c as *Int; }
  unsafe { pd = &d as *Int; }
  unsafe { va = *pa; }
  unsafe { vb = *pb; }
  unsafe { vc = *pc; }
  unsafe { vd = *pd; }
  var sum: Int = va + vb + vc + vd;
  if sum != 100 { return 1; }
  return 0;
}
