// M36-C22: Every pointer pattern -- null pointer, deref, pointer arithmetic, pointer chain, pointer cast, pointer in struct
type Node = { val: Int; next: *Node; }
fn ptr_null() -> Bool {
  var p: *Int;
  unsafe { p = 0 as *Int; }
  if p == unsafe { 0 as *Int } { return true; }
  return false;
}
fn ptr_deref() -> Int {
  var x: Int = 42;
  var p: *Int;
  unsafe { p = &x as *Int; }
  var v: Int;
  unsafe { v = *p; }
  return v;
}
fn ptr_arithmetic() -> Int {
  var a: Int = 10;
  var b: Int = 20;
  var pa: *Int;
  var pb: *Int;
  unsafe { pa = &a as *Int; pb = &b as *Int; }
  if unsafe { pa != pb } { return 0; }
  return 1;
}
fn ptr_chain() -> Int {
  var x: Int = 5;
  var p1: *Int;
  unsafe { p1 = &x as *Int; }
  var p2: *Int = p1;
  var v: Int;
  unsafe { v = *p2; }
  return v;
}
fn ptr_cast() -> Int {
  var x: Int = 99;
  var p: *Int;
  unsafe { p = &x as *Int; }
  var addr: Int;
  unsafe { addr = p as Int; }
  if addr != 0 { return 99; }
  return 0;
}
fn ptr_in_struct() -> Int {
  var n = Node{ val: 42; next: unsafe { 0 as *Node }; };
  return n.val;
}
fn main() -> Int {
  if !ptr_null() { return 1; }
  if ptr_deref() != 42 { return 2; }
  if ptr_arithmetic() != 0 { return 3; }
  if ptr_chain() != 5 { return 4; }
  if ptr_cast() != 99 { return 5; }
  if ptr_in_struct() != 42 { return 6; }
  var x: Int = 7;
  var px: *Int;
  unsafe { px = &x as *Int; }
  var vx: Int;
  unsafe { vx = *px; }
  if vx != 7 { return 7; }
  var nullp: *Int = unsafe { 0 as *Int };
  if nullp != unsafe { 0 as *Int } { return 8; }
  return 0;
}
