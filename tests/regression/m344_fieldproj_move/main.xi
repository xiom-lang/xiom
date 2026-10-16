// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m344 (REL-20261011-1046-engine): a field PROJECTION passed by value
// consumes the projected place -- `take(a.f)` twice is a use-after-move.
// Pre-fix the borrow pass consumed only bare identifiers, so both shapes
// below compiled with ZERO diagnostics. The field's move-onlyness comes
// from its declared type in the struct declaration.
module m344_fieldproj_move;

type Inner = { x: Int; }
type S = { f: Inner; }
type Holder = { fb: Vec[UInt8]; }

fn take(i: Inner) -> Int { return i.x; }
fn take_len(v: Vec[UInt8]) -> Int { return v.len() as Int; }

fn main() -> Int {
  // (1) struct-typed field projection, reused: E001 on 'a.f'.
  var a = S { f: Inner { x: 1 } };
  let r1 = take(a.f);
  let r2 = take(a.f);
  if r1 != 1 { return 1; }
  if r2 != 1 { return 2; }

  // (2) engine shape: a Vec[UInt8] field (dev.fb) reused after a consuming
  // pass: E001 on 'dev.fb'.
  var dev = Holder { fb: Vec[UInt8].new() };
  dev.fb.push(1);
  let n1 = take_len(dev.fb);
  let n2 = take_len(dev.fb);
  if n1 != 1 { return 3; }
  if n2 != 1 { return 4; }
  return 0;
}
