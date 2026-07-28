// M35-T25: Pointer exhaustive — deref, compare
fn deref_int(ptr: *Int) -> Int { var v: Int; unsafe { v = *ptr; } return v; }
fn ptr_struct_deref(ptr: *Int) -> Int { var v: Int; unsafe { v = *ptr; } return v * 2; }
fn ptr_compare(a: *Int, b: *Int) -> Bool { var va: Int; unsafe { va = *a; } var vb: Int; unsafe { vb = *b; } return va == vb; }
fn main() -> Int {
  var x: Int = 42;
  var px: *Int;
  unsafe { px = &x as *Int; }
  if deref_int(px) != 42 { return 1; }
  if ptr_struct_deref(px) != 84 { return 2; }
  var y: Int = 42;
  var py: *Int;
  unsafe { py = &y as *Int; }
  if !ptr_compare(px, py) { return 3; }
  var z: Int = 99;
  var pz: *Int;
  unsafe { pz = &z as *Int; }
  if ptr_compare(px, pz) { return 4; }
  return 0;
}

