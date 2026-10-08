// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m225 (C-ORBIT-04): twin of a_enum with SWAPPED variant order.
module b_enum

pub enum BKind {
  Update,
  Insert,
  Delete,
}

pub fn b_make() -> BKind {
  return Update;
}

pub fn b_code(k: BKind) -> Int {
  match k {
    Insert => { return 1; }
    Update => { return 0; }
    Delete => { return 2; }
  }
}
