// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Confined-fault repeat lock. The POSIX trampoline saves no signal mask on
// the hot path (savemask=0); the handler must restore the delivered signal's
// mask on a fault, or a SECOND fault in the same thread would terminate the
// process instead of being contained. Both faults must surface through the
// typed fault record (exit 70 while unacknowledged).
module confined_repeat_fault.main

use xiom.io;

fn fault_once() -> Int
  requires: true
{
  var p: *Int;
  unsafe { p = 0 as *Int; }
  var x: Int = 42;
  unsafe { x = *p; }
  return x;
}

fn main() -> Int {
  let a = fault_once();
  let b = fault_once();
  io.println("a=" + a.to_str() + " b=" + b.to_str());
  return 0;
}
