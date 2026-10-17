// m361 defect 1 companion (REL-20261017-1227-packages): the process-wide
// hook-registry shapes that actually hit the ginit thunk path. A
// module-global initializer that stores a bare fn through a fn-typed
// destination (`Box{ f: inc_r }`) or passes one to a fn-typed param
// (`apply(inc, 41)`) defers a __fnwrap_N forwarding thunk inside the
// synthesized @__xiom_ginit body. Pre-fix that list was never flushed, so
// clang rejected "use of undefined value '@__fnwrap_N'".
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module probe_global_registry

type R = { a: Int; }
type Box = { f: fn(Int) -> R; }
type F = fn(Int) -> Int;

fn inc(x: Int) -> Int { return x + 1; }
fn inc_r(x: Int) -> R { return R{ a: x + 1 }; }
fn apply(f: fn(Int) -> Int, x: Int) -> Int { return f(x); }

var plain: fn(Int) -> Int = inc;
var box: Box = Box{ f: inc_r };
var seed: Int = apply(inc, 41);
var alias_hook: F = inc;

pub fn main() -> Int {
  if plain(1) != 2 { return 1; }
  // Bind the field first: the checker types a DIRECT global-struct field
  // call as Unit today (recorded residual in COMPILER_BUGS m361); the
  // binding exercises the same ENV store + signature record.
  let bf = box.f;
  if bf(4).a != 5 { return 2; }
  if seed != 42 { return 3; }
  if alias_hook(1) != 2 { return 4; }
  return 0;
}

