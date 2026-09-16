// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// R9 regression (stdlib report 2026-09-12): a FULL-PATH call into a module
// that was never imported must still resolve that shim's own delegated
// full-path call. `xiom.string.glob.glob_match` is a shim whose body calls
// `xiom.misc.glob.glob_match`; pre-fix the target module was never injected
// (peek is non-caching and its `use` deps were not followed), so codegen
// bound the inner call to the shim itself -> infinite recursion ->
// 0xC0000409 at runtime. Repro probes: p_x1/p_x2/p_x4/p_x6,
// p_sdx_shim_first, p_lev_shim_first.
module m70_full_path_shim_delegation
use xiom.convert;

fn main() -> Int {
  if !xiom.string.glob.glob_match("*.xi", "main.xi") { return 1; }
  if !xiom.string.glob.glob_match("a?c", "abc") { return 2; }
  if xiom.string.glob.glob_match("*.rs", "main.xi") { return 3; }
  if !xiom.string.glob.glob_match_case_insensitive("A?C", "abc") { return 4; }
  if xiom.string.glob.glob_match_case_insensitive("A?C", "xyz") { return 5; }
  return 0;
}
