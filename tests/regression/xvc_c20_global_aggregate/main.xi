// XVC-C-20 minimal (REL-20261018-1815-xvector): `&mut` on a module-level
// aggregate must alias the global, not a temporary copy. Pre-fix the callee
// increments a copy and the global stays 0.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module xvc_c20_global_aggregate

use xiom.io;
use xiom.convert;

type Engine = { count: Int; tag: Int; }

var G: Engine = Engine{ count: 0; tag: 7; };

fn bump(e: &mut Engine) -> Int {
  e.count = e.count + 1;
  return e.count;
}

fn read_count(e: &Engine) -> Int {
  return e.count;
}

fn main() -> Int {
  let a = bump(&mut G);
  let b = read_count(&G);
  io.println("a=" + convert.int_to_string(a) + " b=" + convert.int_to_string(b));
  if a != 1 { return 1; }
  if b != 1 { return 2; }
  if G.count != 1 { return 3; }
  if G.tag != 7 { return 4; }
  return 0;
}
