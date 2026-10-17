// Small-aggregate C ABI probe: an 8-byte record is passed/returned by C in a
// SINGLE i64 register. LLVM's naked %struct lowering splits the two i32
// fields into ecx/edx (call) and eax/edx (return), which C does not read the
// same way; m351 coerces the raw bytes to/from i64.
// Expected: c6 17036 / c7 507,509.
// Parameterized by XIOM_M351_PROBE_DLL (see probe_ladder.xi).
//
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module probe_small

use xiom.ffi.dl;
use xiom.convert;
use xiom.io;
use xiom.os.env;

type Pair2i = {
  a: Int32;
  b: Int32;
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
  let p = Pair2i{ a: 17; b: 29 };

  let s2i = dl.dl_sym(handle, "probe_sum2i");
  if s2i.is_ok {
    say("c6 8B-record-by-value: calling");
    unsafe {
      let f = s2i.value as fn(Pair2i, Int) -> Int;
      let r = f(p, 7);
      say("c6 returned " + convert.int_to_string(r));
    }
  }

  let m2i = dl.dl_sym(handle, "probe_make2i");
  if m2i.is_ok {
    say("c7 8B-record-return: calling");
    unsafe {
      let f = m2i.value as fn(Int) -> Pair2i;
      let r = f(500);
      say("c7 fields " + convert.int_to_string(r.a as Int) + "," + convert.int_to_string(r.b as Int));
    }
  }

  say("done");
  return 0;
}
