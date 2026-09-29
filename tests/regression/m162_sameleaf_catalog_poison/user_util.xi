// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m162 poison half: a user module exporting a fn whose LEAF matches a stdlib
// fn. Merely being in the import closure used to poison the catalog-body
// check of UNRELATED stdlib modules (bogus T001s in [xiom.num]) because the
// first-wins global bare slot belonged to this module.
module user_util

pub fn char_at(s: Str, pos: Int) -> Int {
  return 7;
}
