// Direct `extern "C"` twin (not a fn-cast): a 16B record BY VALUE. Pre-fix
// the declare/call were naked (`%struct.Pair2` in rcx:rdx) -> the C callee
// dereferenced 17 as a pointer. Post-fix: 17036. Build with `--c-source probe.c`.
//
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module probe_extern_direct

use xiom.convert;
use xiom.io;

type Pair2 = {
  a: Int;
  b: Int;
}

extern "C" {
  fn probe_sum2(p: Pair2, bias: Int) -> Int;
}

fn say(s: Str) {
  io.println(s);
  io.flush_stdout();
}

pub fn main() -> Int {
  let p = Pair2{ a: 17; b: 29 };
  say("direct extern 16B-record-by-value: calling");
  var r = 0;
  unsafe {
    r = probe_sum2(p, 7);
  }
  say("direct extern returned " + convert.int_to_string(r));
  return 0;
}
