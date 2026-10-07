// m204 (C-PULSE-07): a module-level `var` initialized by a cross-module
// constructor call must emit the callee body. Pre-fix the checker's
// reachability filter ignored TopDecl::Const initializers, so the catalog fn
// referenced ONLY from the global init was pruned and codegen emitted
// `call i64 @rate_keyed_new` with no definition (clang "use of undefined
// value '@rate_keyed_new'"; Pulse: xiom.rate limiter).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m204_pkginit

use xiom.rate;

var bucket = rate_keyed_new(20, 22);

fn main() -> Int {
  // The defect was a COMPILE-stage failure (the callee body was pruned:
  // `call i64 @rate_keyed_new` with no definition). Reading `bucket` here
  // hits a separate checker typing gap for cross-module-initialized globals
  // (m204 follow-up), so this lock only requires codegen + run.
  return 0;
}
