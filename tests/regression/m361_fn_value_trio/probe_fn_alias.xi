// Probe (REL-20261017-1227-packages item 3, xiom.pulse 0.2 hook seam):
// a fn-typed alias used as a callable parameter is accepted by the type
// checker but miscompiled at codegen/runtime (0.64.6). The same call with
// the direct fn type in the signature works.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module probe_fn_alias

pub type F = fn(Int) -> Int;

fn inc(x: Int) -> Int { return x + 1; }

fn install(v: &mut Vec[fn(Int) -> Int], f: F) {
  v.push(f);
}

fn apply_alias(f: F, x: Int) -> Int {
  return f(x);
}

pub fn main() -> Int {
  var v: Vec[fn(Int) -> Int] = Vec[fn(Int) -> Int].new();
  install(&mut v, inc);
  let f = v[0];
  let n: Int = f(4);
  if n != 5 { return 10; }

  // m361: an alias param CALLED in the callee body (env-first path).
  if apply_alias(inc, 4) != 5 { return 11; }

  // m361: an alias-typed local binding keeps the fn signature record.
  let g: F = inc;
  if g(4) != 5 { return 12; }
  return 0;
}
