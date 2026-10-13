// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m288 lock (REL-20261010-1548-stdlib-4): `x.a@pre` on a `&mut` struct
// PARAMETER must snapshot the entry value. The v0.64.4 archive binary
// evaluated it against the LIVE post-mutation field (the clause read 2
// after bumping 1 -> 2 and reported "contract violated: ensures at
// 23:12", rc 1); HEAD is green and enforcement is verified live (a body
// mutated to +5 still violates at runtime). Self-field @pre on method
// receivers was already correct; this pins the &mut-parameter half.
module m288_mut_param_field_pre.main

type Pair = { a: Int; b: Int; }

fn bump(x: &mut Pair) -> Int
  ensures: (x.a@pre < 100) => (x.a == x.a@pre + 1)
{
  x.a = x.a + 1;
  return x.a;
}

fn main() -> Int {
  var p = Pair{ a: 1; b: 0; };
  let v = bump(&p);
  if v != 2 { return 1; }
  if p.a != 2 { return 2; }
  return 0;
}
