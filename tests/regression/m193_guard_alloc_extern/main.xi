// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m193: a direct extern of the runtime symbol xiom_guard_alloc must compile
// and link. Pre-fix the compiler emitted the builtin declare plus the user's
// declare and clang rejected the module ("invalid redefinition of function
// 'xiom_guard_alloc'"). Outside a confined region xiom_guard_alloc falls
// back to xiom_alloc, so the address is non-null.
use xiom.io;

extern "C" {
  fn xiom_guard_alloc(size: Int) -> *UInt8;
}

fn main() -> Int {
  var addr: Int = 0;
  unsafe {
    addr = xiom_guard_alloc(16) as Int;
  }
  if addr == 0 { return 1; }
  io.println("m193 ok");
  return 0;
}
