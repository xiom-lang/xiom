// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// C22 lock: a sibling-module import AND a packages/ import resolved
// relative to the SCRIPT'S directory must work under `xiom run` exactly as
// they do under `xiom --check`. `xiom run` compiles a temp copy under
// <tmp>/xiom_run, so without the source-dir hint the catalog never sees
// this directory.
module c22_run_sibling_main

use c22pkg.hello;
use c22_sibling_lib;

fn main() -> Int {
  if c22_sibling_lib.bump(41) != 42 { return 1; }
  if hello.hi() != 4242 { return 2; }
  return 0;
}
