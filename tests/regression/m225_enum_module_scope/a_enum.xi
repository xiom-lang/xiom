// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m225 (C-ORBIT-04): enum variant construction must be scoped to the DECLARING
// module. Two modules can declare structurally identical enums (`Insert`/
// `Update`); injected enums are keyed bare in codegen, so the first-registered
// twin won and `return Update;` in b_enum built a_enum's type -- clang rejected
// the module ("ret type %struct.BKind vs %struct.AKind"). The two enums below
// deliberately order their shared variants DIFFERENTLY so a wrong parent also
// flips discriminant tags, catching silent mis-binding.

module a_enum

pub enum AKind {
  Insert,
  Update,
}

pub fn a_make() -> AKind {
  return Update;
}

pub fn a_code(k: AKind) -> Int {
  match k {
    Insert => { return 1; }
    Update => { return 0; }
  }
}
