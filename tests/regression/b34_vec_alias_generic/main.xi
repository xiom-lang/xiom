// B-34 minimal (REL-20261017-0042-bindings): a module-local alias in a
// generic position (`&Vec[Handle]`, Handle = Int) must unify with Vec[Int]
// across modules. Pre-fix this was T001 "expected Vec[Handle], found
// Vec[Int]" before codegen.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module b34_probe

use d_mod;
use xiom.io;
use xiom.convert;

fn main() -> Int {
  var v: Vec[Int] = Vec[Int].new();
  v.push(7);
  let n = d_mod.take_handles(&v);
  let h = d_mod.make_handle();
  io.println("n=" + convert.int_to_string(n) + " h=" + convert.int_to_string(h));
  if n != 1 { return 1; }
  if h != 42 { return 2; }
  return 0;
}
