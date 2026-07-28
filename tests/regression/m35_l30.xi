// M35-L30: Pointer iteration — iterate over adjacently allocated vars via pointer
fn get_next(p: *Int, stride: Int) -> *Int {
  var addr: Int;
  unsafe { addr = p as Int; }
  var next: *Int;
  unsafe { next = (addr + stride) as *Int; }
  return next;
}

fn main() -> Int {
  var a: Int = 10;
  var b: Int = 20;
  var c: Int = 30;
  var pa: *Int;
  var pc: *Int;
  unsafe { pa = &a as *Int; }
  unsafe { pc = &c as *Int; }
  var va: Int;
  var vc: Int;
  unsafe { va = *pa; }
  unsafe { vc = *pc; }
  if va != 10 { return 1; }
  if vc != 30 { return 2; }
  return 0;
}
