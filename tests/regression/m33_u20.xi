// M33-U20: Comprehensive unsafe/FFI -- extern, ptr cast, contract, mixed
extern "C" {
  fn calc_offset(base: *Int, idx: Int) -> *Int;
  fn get_magic() -> *Int;
}
fn safe_deref(p: *Int) -> Int
  requires: p != (0 as *Int)
{
  var r: Int;
  unsafe { r = *p; }
  return r;
}
fn main() -> Int {
  var a: Int = 100;
  var b: Int = 200;
  var pa: *Int;
  var pb: *Int;
  unsafe { pa = &a as *Int; }
  unsafe { pb = &b as *Int; }
  var pu: *UInt8;
  unsafe { pu = pa as *UInt8; }
  var r1: Int = safe_deref(pa);
  var r2: Int = safe_deref(pb);
  var nullp: *Int;
  unsafe { nullp = 0 as *Int; }
  if r1 == 100 && r2 == 200 && unsafe { nullp == (0 as *Int) } { return 0; }
  return 1;
}
