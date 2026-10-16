// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m344 (REL-20261011-1046-engine): a local bound to a CALL RESULT must be
// movable by the E001 pass. Pre-fix the borrow checker was type-blind (the
// "Int" fallback made call-init locals Copy) and the reuse below compiled
// with ZERO diagnostics, even under --strict. The annotated twin (b) warned
// even pre-fix -- this fixture locks annotated/unannotated parity.
module m344_callinit_move;

type S = { v: Int; }

fn make() -> S { return S { v: 1 }; }
fn take(s: S) -> Int { return s.v; }

fn main() -> Int {
  // (1) unannotated call-init local, reused as a bare argument: E001 on the
  // second use. Warning by default (the ABI still passes a copy, so the
  // values stay readable at runtime); hard error under --strict.
  var a = make();
  let r1 = take(a);
  let r2 = take(a);
  if r1 != 1 { return 1; }
  if r2 != 1 { return 2; }

  // (2) annotated control: same diagnostics as (1) -- parity.
  var b: S = make();
  let r3 = take(b);
  let r4 = take(b);
  if r3 != 1 { return 3; }
  if r4 != 1 { return 4; }
  return 0;
}
