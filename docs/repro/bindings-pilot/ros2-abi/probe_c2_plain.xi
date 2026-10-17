// ABI probe control: 16B record BY VALUE through an fn-cast, NO surrounding
// unsafe block. This isolates the C ABI from the confinement path: pre-fix it
// hard-crashed with 0xC0000005 (loud); post-fix it must return 17036.
// Parameterized by XIOM_M351_PROBE_DLL (see probe_ladder.xi).
//
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module probe_c2_plain

use xiom.ffi.dl;
use xiom.convert;
use xiom.io;
use xiom.os.env;

type Pair2 = {
  a: Int;
  b: Int;
}

fn say(s: Str) {
  io.println(s);
  io.flush_stdout();
}

pub fn main() -> Int {
  let dll = env.get_var("XIOM_M351_PROBE_DLL");
  if !dll.is_ok {
    say("XIOM_M351_PROBE_DLL is not set");
    return 2;
  }
  let h = dl.dl_open(dll.value);
  if !h.is_ok {
    say("load failed: " + h.error);
    return 1;
  }
  let handle = h.value;
  let p = Pair2{ a: 17; b: 29 };

  let s2 = dl.dl_sym(handle, "probe_sum2");
  if s2.is_ok {
    say("c2 16B-record-by-value (no unsafe): calling");
    let f = s2.value as fn(Pair2, Int) -> Int;
    let r = f(p, 7);
    say("c2 returned " + convert.int_to_string(r));
  }

  say("done");
  return 0;
}
