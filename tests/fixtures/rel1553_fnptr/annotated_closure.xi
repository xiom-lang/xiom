// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// REL-20261010-1553 probe (b): an annotated closure-literal binding.
// track_fn_value_binding removes closure_locals when sig_marker.is_some(), so
// the call inttoptrs the env as code -> 0xC0000005.
module annotated_closure.main

use xiom.io;

fn main() -> Int
{
  let f: fn(Int) -> Int = |x| x + 1;
  let r = f(3);
  if r != 4 {
    io.println("r=" + r.to_str());
    return 1;
  }
  io.println("annotated closure ok");
  return 0;
}
