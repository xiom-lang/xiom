// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m225 (C-ORBIT-04): caller -- both twin enums in one compilation unit.
module m225_enum_module_scope

use a_enum;
use b_enum;

fn main() -> Int {
  // Unqualified construction inside each module's own fn must bind that
  // module's enum; the codes prove the right discriminant tags were used.
  if a_enum.a_code(a_enum.a_make()) != 0 { return 1; }
  if b_enum.b_code(b_enum.b_make()) != 0 { return 2; }

  // Caller-side match on the returned value resolves through the scrutinee's
  // declared type (BKind), not the first-declared twin.
  let b = b_enum.b_make();
  match b {
    Insert => { return 3; }
    Update => { return 0; }
    Delete => { return 4; }
  }
}
