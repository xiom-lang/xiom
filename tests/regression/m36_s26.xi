// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-S26: Virtual machine -- stack push/pop and arithmetic operations
type StackFrame = { sp: Int; bp: Int; ip: Int; }
type StackVal = { tag: Int; int_val: Int; }
fn make_frame(sp: Int, bp: Int, ip: Int) -> StackFrame {
  return StackFrame{ sp: sp; bp: bp; ip: ip; };
}
fn make_val(tag: Int, iv: Int) -> StackVal {
  return StackVal{ tag: tag; int_val: iv; };
}
fn is_int(v: StackVal) -> Bool { return v.tag == 0; }
fn is_float(v: StackVal) -> Bool { return v.tag == 1; }
fn is_ptr(v: StackVal) -> Bool { return v.tag == 2; }
fn push_val(sp: Int) -> Int { return sp + 1; }
fn pop_val(sp: Int) -> Int {
  if sp <= 0 { return 0; }
  return sp - 1;
}
fn stack_add(a: StackVal, b: StackVal) -> StackVal {
  if !is_int(a) || !is_int(b) { return make_val(-1, 0); }
  return make_val(0, a.int_val + b.int_val);
}
fn stack_mul(a: StackVal, b: StackVal) -> StackVal {
  if !is_int(a) || !is_int(b) { return make_val(-1, 0); }
  return make_val(0, a.int_val * b.int_val);
}
fn stack_cmp_eq(a: StackVal, b: StackVal) -> Bool {
  return a.int_val == b.int_val && a.tag == b.tag;
}
fn main() -> Int {
  var f = make_frame(0, 0, 0);
  if f.sp != 0 || f.bp != 0 { return 1; }
  var v = make_val(0, 42);
  if !is_int(v) { return 2; }
  if is_float(v) { return 3; }
  if is_ptr(v) { return 4; }
  var sp1 = push_val(0);
  if sp1 != 1 { return 5; }
  var sp2 = push_val(sp1);
  if sp2 != 2 { return 6; }
  var sp3 = pop_val(sp2);
  if sp3 != 1 { return 7; }
  var sp0 = pop_val(sp3);
  if sp0 != 0 { return 8; }
  var sp0b = pop_val(0);
  if sp0b != 0 { return 9; }
  var a = make_val(0, 10);
  var b = make_val(0, 20);
  var sum = stack_add(a, b);
  if sum.int_val != 30 || sum.tag != 0 { return 10; }
  var prod = stack_mul(a, b);
  if prod.int_val != 200 { return 11; }
  if !stack_cmp_eq(a, make_val(0, 10)) { return 12; }
  if stack_cmp_eq(a, b) { return 13; }
  return 0;
}
