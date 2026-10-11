// m267 (REL-20261010-1802-orbitdb, twin half): a foreign same-named type in
// the program must not turn a scope-correct match into a hard T001. The
// scrutinee's enum owns its BARE pattern variants; the local DbError only
// shares the leaf names. Pre-fix: "match pattern 'Corruption' belongs to
// 'DbError', not 'CoreError'". Post-fix this compiles and the match runs
// against twin.CoreError's tags. Exit 0 on success.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m267_twin_scrutinee_enum

use twin;

pub enum DbError {
  NotFound,
  Corruption,
}

fn fold(e: &CoreError) -> Int {
  match e {
    NotFound => { return 0; }
    Corruption(_) => { return 1; }
    InvalidInput(_) => { return 2; }
  }
}

fn main() -> Int {
  let a = CoreError.NotFound;
  let b = CoreError.Corruption("x");
  let c = CoreError.InvalidInput("y");
  if fold(&a) != 0 {
    return 1;
  }
  if fold(&b) != 1 {
    return 2;
  }
  if fold(&c) != 2 {
    return 3;
  }
  return 0;
}
