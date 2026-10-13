// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m298 "engine" module (xvector role): a struct whose Counter field resolves
// through the shim import. Post-fix the bare reference binds demo.shim.Counter
// (the unique imported owner); pre-fix it happened to be the shortest
// same-leaf key too, so this module is the green control of the lock.
module demo.engine;

use demo.shim;

pub type Eng = {
  upserts: Counter;
}

pub fn eng_new() -> Eng {
  return Eng{ upserts: counter_new("eng.upserts") };
}

pub fn eng_up(e: &mut Eng) {
  counter_inc(&mut e.upserts);
}
