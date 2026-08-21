// M35-L22: Pointer to pointer -- double indirection via pointer chain
fn deref_int(p: *Int) -> Int {
  var v: Int;
  unsafe { v = *p; }
  return v;
}

fn main() -> Int {
  var x: Int = 77;
  var p: *Int;
  var q: *Int;
  unsafe { p = &x as *Int; }
  unsafe { q = p as *Int; }
  if deref_int(p) != 77 { return 1; }
  if deref_int(q) != 77 { return 2; }
  return 0;
}
