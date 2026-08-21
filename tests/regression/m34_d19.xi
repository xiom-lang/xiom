// M34-D19: Diamond-shaped type graph -- A->B,C; B,C->D forming diamond dependency
type D = { label: Int; }
type B = { bval: Int; ref: *D; }
type C = { cval: Int; ref: *D; }
type A = { aval: Int; left: *B; right: *C; }

fn read_b_val(b: *B) -> Int {
  if b == (unsafe { 0 as *B }) { return -1; }
  var v: Int;
  unsafe { v = (*b).bval; }
  return v;
}

fn read_c_val(c: *C) -> Int {
  if c == (unsafe { 0 as *C }) { return -1; }
  var v: Int;
  unsafe { v = (*c).cval; }
  return v;
}

fn read_d_label(d: *D) -> Int {
  if d == (unsafe { 0 as *D }) { return -1; }
  var v: Int;
  unsafe { v = (*d).label; }
  return v;
}

fn read_a_val(a: *A) -> Int {
  if a == (unsafe { 0 as *A }) { return -1; }
  var v: Int;
  unsafe { v = (*a).aval; }
  return v;
}

fn diamond_null_check(a: *A) -> Bool {
  if a == (unsafe { 0 as *A }) { return true; }
  var b: *B;
  var c: *C;
  unsafe { b = (*a).left; }
  unsafe { c = (*a).right; }
  return b == (unsafe { 0 as *B }) && c == (unsafe { 0 as *C });
}

fn main() -> Int {
  var da: *A = unsafe { 0 as *A };
  var db: *B = unsafe { 0 as *B };
  var dc: *C = unsafe { 0 as *C };
  var dd: *D = unsafe { 0 as *D };
  if read_a_val(da) != -1 { return 1; }
  if read_b_val(db) != -1 { return 2; }
  if read_c_val(dc) != -1 { return 3; }
  if read_d_label(dd) != -1 { return 4; }
  if !diamond_null_check(da) { return 5; }
  return 0;
}
