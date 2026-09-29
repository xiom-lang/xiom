// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m162 lock (playground relay): a user module exporting a same-leaf fn
// (`char_at`) must not poison the catalog-body check of `xiom.num`, whose own
// `use xiom.string.char_at` is the resolution that matters inside its body.
// Before the fix, bare-call resolution inside a catalog body hit the global
// first-wins bare slot (owned by the user module, which registers first):
// num's `char_at` returned Int instead of Option[Char] -> bogus T001s
// ("cannot access field on non-struct type Int") aborted codegen.
module m162_sameleaf_catalog_poison

use xiom.num;
use user_util;

fn main() -> Int {
  // The user's own same-leaf fn still resolves for the user.
  if user_util.char_at("x", 0) != 7 { return 1; }

  // Catalog body (xiom.num -> xiom.string.char_at) must resolve cleanly.
  match num.parse_int("42") {
    Ok(v) => { if v != 42 { return 2; } }
    Err(e) => { return 3; }
  }
  match num.parse_int_radix("ff", 16) {
    Ok(v) => { if v != 255 { return 4; } }
    Err(e) => { return 5; }
  }
  return 0;
}
