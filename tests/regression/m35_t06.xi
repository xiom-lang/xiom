// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T06: Unit/void returns -- functions returning nothing, calling void functions
fn nop1() { return; }
fn nop2() { return; }
fn nop3() { var x = 1; x = x + 1; return; }
fn void_if(flag: Bool) { if flag { nop1(); return; } nop2(); return; }
fn void_while(n: Int) { var i = 0; while i < n { nop1(); i = i + 1; } return; }
fn void_match(v: Int) { match v { 1 => nop1(), 2 => nop2(), _ => nop3() }; }
fn void_generic[T](x: T) { return; }
fn void_ptr() { var x = 42; var p: *Int; unsafe { p = &x as *Int; } return; }
fn main() -> Int {
  nop1(); nop2(); nop3();
  void_if(true); void_if(false);
  void_while(3);
  void_match(1); void_match(2); void_match(99);
  void_generic(42); void_generic("hi"); void_generic(true);
  void_ptr();
  return 0;
}

