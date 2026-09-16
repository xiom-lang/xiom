// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M62 (stdlib-audit #3 delegation crash): a module-qualified call MUST bind
// the catalog fn even when a local fn with the same bare name exists.
// The checker's decl injection used to SKIP any stdlib free fn whose bare
// name matched a user fn (user_free_fns shadow set), so the leaf-qualified
// key never registered; codegen's resolve_module_call then fell through to
// the bare name and bound the LOCAL fn (silent wrong-module resolution;
// historically the 0xC0000005 delegation crash that forced the stdlib's
// copy-paste base64/base58 duplicates). Fixed: injection dedups by the
// QUALIFIED key only; injected decls emit leaf-qualified symbols
// (@convert.to_base58) while the user's fn keeps the bare symbol.
module m62_delegation_shadow

use xiom.string;

pub fn str_upper(s: Str) -> Str {
  return "LOCAL";
}

fn main() -> Int {
  // qualified -> catalog fn ("ABC")
  var a = xiom.string.str_upper("abc");
  // bare -> local shadow ("LOCAL")
  var b = str_upper("abc");
  if a != "ABC" { return 1; }
  if b != "LOCAL" { return 2; }
  return 0;
}
