// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m298 lock (REL-20261010-1709-xvector): two modules each declare a Counter
// and a third module's struct field (`hits: Counter`) resolves through a
// `use demo.metrics;` import. Pre-fix the injected bare `Counter` reference
// stayed unqualified, so the type-decl emission used codegen's deterministic
// same-leaf suffix pick (shortest key -> demo.shim.Counter) while
// metric_counter_new() returned demo.metrics.Counter -- clang rejected the
// store ('%tmp defined with type ... but expected ...'), the xvector
// xv-counter-type-shadow witness in miniature. Pre-fix binary: compile fails
// at clang; post-fix the unique import owner wins.
module m298_xvector_counter_shadow;

use xiom.io;
use xiom.convert;
use demo.engine;
use demo.tally;

fn main() -> Int {
  var e = eng_new();
  eng_up(&mut e);
  var t = tally_new();
  tally_hit(&mut t);
  let sv = e.upserts.value;
  let th = tally_hits(&t);
  if sv != 1 || th != 1 {
    io.println("m298_xvector_counter_shadow: [CORRUPT] shim=" + convert.int_to_string(sv)
      + " tally=" + convert.int_to_string(th));
    return 1;
  }
  io.println("shim counter=" + convert.int_to_string(sv)
    + " tally hits=" + convert.int_to_string(th));
  return 0;
}
