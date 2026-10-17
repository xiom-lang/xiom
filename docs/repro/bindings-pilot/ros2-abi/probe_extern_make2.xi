// Direct `extern "C"` 16B record RETURN: pre-fix `call %struct.Pair2` read
// rax:rdx while C used an sret pointer -> silent exit 0 inside a confined
// block. Post-fix: 507,509. Build with `--c-source probe.c`.
//
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module probe_extern_make2

use xiom.convert;
use xiom.io;

type Pair2 = {
  a: Int;
  b: Int;
}

extern "C" {
  fn probe_make2(base: Int) -> Pair2;
}

fn say(s: Str) {
  io.println(s);
  io.flush_stdout();
}

pub fn main() -> Int {
  say("direct extern 16B-record-return: calling");
  var r = Pair2{ a: 0; b: 0 };
  unsafe {
    r = probe_make2(500);
  }
  say("direct extern fields " + convert.int_to_string(r.a) + "," + convert.int_to_string(r.b));
  return 0;
}
