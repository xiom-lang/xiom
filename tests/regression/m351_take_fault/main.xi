// m351 (REL-20261016-2031-bindings-3, B-33): typed-fault acknowledgement.
// A deterministic AV inside a confined block must be CAUGHT, manifested with
// an always-on [fault] diagnostic, and acknowledged through
// xiom.error.take_fault() -- the only exit-0 path after a fault.
//
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m351_take_fault

use xiom.convert;
use xiom.error;
use xiom.io;

extern "C" {
  fn xiom_fault_av() -> Int;
}

fn main() -> Int {
  var r = 0;
  unsafe {
    r = xiom_fault_av();
  }
  if r != 0 { return 1; }
  if !error.faulted() { return 2; }
  match error.take_fault() {
    Some(f) => {
      io.println("fault acknowledged: " + f.signal + " pc=" + convert.int_to_string(f.pc as Int));
    }
    None => { return 3; }
  }
  if error.faulted() { return 4; }
  io.println("TAKE-FAULT-OK");
  return 0;
}
