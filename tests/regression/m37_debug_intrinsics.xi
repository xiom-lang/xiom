// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_debug_intrinsics
// BUG 27: in-code debug intrinsics -- dbg!(expr) (print + return the value),
// assert(cond[, "msg"]) (runtime-checked invariant, clean panic on
// violation), debugger; (break into an attached debugger -- no-op without
// one). todo!()/unimplemented!() panic with the source location (verified
// manually -- they exit 1 by design).

use xiom.io;

fn main() -> Int {
  var x = 5;
  var y = dbg!(x + 2);
  if y != 7 { return 1; }
  var d = dbg!(2.5);
  if d != 2.5 { return 2; }
  var s = dbg!("str");
  if s != "str" { return 3; }
  assert(x == 5);
  assert(x == 5, "x must be five");
  debugger;
  io.println("debug ok");
  return 0;
}
