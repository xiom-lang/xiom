// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m219 (XVC-C-01): calling a `&fn() -> T` parameter. Pre-fix `f()` emitted
// `inttoptr i64* %slot to i64 ()*` (invalid cast from ptr to ptr; clang exit
// 1), `(*f)()` fell into the M20-A1 closure fallback with a bogus 1-arg
// signature and crashed, and `var g = *f` loaded the function's first
// instructions instead of the code address (0xC0000005). All three shapes
// must return 7.

module m219_fnptr_ref_call

fn seven() -> Int { return 7; }

fn call_direct(f: &fn() -> Int) -> Int {
  return f();
}

fn call_deref(f: &fn() -> Int) -> Int {
  return (*f)();
}

fn call_local(f: &fn() -> Int) -> Int {
  var g: fn() -> Int = *f;
  return g();
}

fn main() -> Int {
  if call_direct(&seven) != 7 { return 1; }
  if call_deref(&seven) != 7 { return 2; }
  if call_local(&seven) != 7 { return 3; }
  return 0;
}
