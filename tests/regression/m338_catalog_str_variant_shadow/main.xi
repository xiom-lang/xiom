// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m338 lock (REL-20261016-1128-engine class 1) -- entry side: the imported
// module declares an enum variant named `Str` and calls `Str::from_utf8`
// inside a struct literal. Pre-fix the imported body fails with T001
// "cannot call 'from_utf8' on this expression" (catalog body
// [xiom.m338x.json_lite]) and compile_and_run returns None; the same shape
// broke the engine's json_lite.xi at 0.64.5 and at the m334 build. Post-fix
// the bytes must be EXACT at runtime: 'A' read back as "A".
module m338_catalog_str_variant_shadow.main;

use xiom.io;
use xiom.m338x.json_lite;

fn main() -> Int {
  match jl_probe(0) {
    Ok(parsed) => {
      var s = jl_str_of(parsed.value);
      if s != "A" {
        io.println("m338: [CORRUPT] from_utf8 bytes = " + s);
        return 1;
      }
      if parsed.pos != 1 {
        io.println("m338: [CORRUPT] pos = " + parsed.pos.to_str());
        return 2;
      }
    }
    Err(e) => {
      io.println("m338: [CORRUPT] probe error = " + e);
      return 3;
    }
  };
  io.println("m338_catalog_str_variant_shadow: ok");
  return 0;
}
