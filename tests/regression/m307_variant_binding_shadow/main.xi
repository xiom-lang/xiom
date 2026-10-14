// m307 lock (REL-20261013-1324-selfhost; correction REL-20261013-1732):
// variant-pattern payload extraction must resolve the CANONICAL field at the
// binding's POSITION. The pattern binding name is a local, not a field key --
// pre-fix a binding named like another variant's field won the direct name
// lookup and read the wrong slot (selfhost: `NkExprField(recv, name_node)`
// read slot 0 = 461 while a canonical accessor read 23; the enum field list
// put NkFn.recv before NkExprField.inner). Both shapes below are red on the
// archived v0.64.4 (IR: read_renamed loads `i32 0, i32 1`) and on the
// pre-fix HEAD. Copyright (c) 2026 Eleftherios Notas and The XIOM Authors --
// SPDX-License-Identifier: MIT OR Apache-2.0
module m307_variant_binding_shadow

use xiom.io;
use xiom.convert;

enum Pair {
  P(first: Int, second: Int),
  Q(marker: Int),
}

// Same-variant swap: binding "second" names a LATER canonical field of the
// same variant. Pre-fix the direct name match reads slot 2 (7), post-fix
// the canonical slot 1 (23). Deterministic -- slot 2 is written.
fn read_swapped(p: Pair) -> Int {
  match p {
    P(second, other) => { return second; }
    _ => { return -1; }
  }
}

enum Kind {
  Wrap(recv: Int),
  Field(inner: Int, name: Int),
}

// Selfhost shape: binding "recv" collides with Wrap.recv at parent field
// index 1; the target variant writes slots 2 ("inner") and 3 ("name") only.
fn read_renamed(k: Kind) -> Int {
  match k {
    Field(recv, name_node) => { return recv; }
    _ => { return -1; }
  }
}

// Canonical-name control (the single-purpose accessor shape).
fn read_canonical(k: Kind) -> Int {
  match k {
    Field(inner, name) => { return inner; }
    _ => { return -1; }
  }
}

fn main() -> Int {
  var bad = 0;

  let p = Pair.P(23, 7);
  let a = read_swapped(p);
  io.println("swapped=" + convert.int_to_string(a) + " (expected 23)");
  if a != 23 { bad = bad + 1; }

  let k = Kind.Field(23, 7);
  let b = read_renamed(k);
  io.println("renamed=" + convert.int_to_string(b) + " (expected 23)");
  if b != 23 { bad = bad + 1; }

  let c = read_canonical(k);
  io.println("canonical=" + convert.int_to_string(c) + " (expected 23)");
  if c != 23 { bad = bad + 1; }

  io.println("failures=" + convert.int_to_string(bad));
  return bad;
}
