// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m340 lock (REL-20261016-1128-engine class 3): a dependency's
// port.args.json C sources are compiled through the rtcache-style
// dependency object cache (crates/xiom/src/rtcache.rs) -- the second
// consumer build reuses the objects instead of recompiling every TU in the
// single clang link invocation (m310 behavior). The helper ships THREE
// --c-source TUs and a `--timeout 600` (exercised by
// e2e_m340_dep_timeout_floor).
module m340_dep_csource_cache.main

use m340_cache_helper.helper;

fn main() -> Int {
  // (x+1) + (x+2) + (x+4) at x=1 -> 10
  let v = helper.sum(1);
  if v != 10 { return 1; }
  return 0;
}
