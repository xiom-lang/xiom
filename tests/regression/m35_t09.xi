// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T09: Pointers in every context -- var, param, return, deref, address-of
fn deref_int(ptr: *Int) -> Int { var v: Int; unsafe { v = *ptr; } return v; }
fn ptr_if(ptr: *Int) -> Int { var v: Int; unsafe { v = *ptr; } if v > 0 { return v; } return 0; }
fn ptr_struct_deref(ptr: *Int) -> Int { var v: Int; unsafe { v = *ptr; } return v * 2; }
fn ptr_compare(a: *Int, b: *Int) -> Bool { var va: Int; unsafe { va = *a; } var vb: Int; unsafe { vb = *b; } return va == vb; }
fn main() -> Int {
  var x: Int = 42;
  var p: *Int;
  unsafe { p = &x as *Int; }
  var v: Int = deref_int(p);
  if v != 42 { return 1; }
  if ptr_if(p) != 42 { return 2; }
  if ptr_struct_deref(p) != 84 { return 3; }
  var y: Int = 42;
  var p2: *Int;
  unsafe { p2 = &y as *Int; }
  if !ptr_compare(p, p2) { return 4; }
  var z: Int = 99;
  var p3: *Int;
  unsafe { p3 = &z as *Int; }
  if ptr_compare(p, p3) { return 5; }
  return 0;
}

