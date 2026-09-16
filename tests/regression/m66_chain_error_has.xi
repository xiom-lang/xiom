// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// smoke_error2 has-mid root-cause regression (2026-09-11).
//
// The catalog module xiom.error.chain registers its type BARE as
// `ChainError`; `Option[ChainError]` (error_chain_pop) registers the
// generated aggregate key `Option__ChainError`, which suffix-matches the
// bare type name. The Vec[Str]-element resolver stopped at the first
// type_meta key matching the suffix, so whenever HashMap order put the
// aggregate key first, `e.messages[i]` inside error_chain_has loaded the
// Str handle through the scalar i64 path, truncated it to a byte and
// compared garbage. `error_chain_has(e, "mid")` was false on those builds
// (the stdlib smoke_error2 flake) and true on others.
//
// The resolver now prefers exact/dot-qualified keys and never stops at a
// matching key that lacks the field, so this is deterministic.
module m66_chain_error_has
use xiom.error.chain;

fn main() -> Int {
  var e = chain.error_chain_new("root");
  e = chain.error_chain_push(e, "mid");
  e = chain.error_chain_push(e, "top");
  if !chain.error_chain_has(e, "mid") { return 1; }
  if chain.error_chain_has(e, "nope") { return 2; }
  if !chain.error_chain_has(e, "top") { return 3; }
  if !chain.error_chain_has(e, "root") { return 4; }
  let popped = chain.error_chain_pop(e);
  match popped {
    Some(pe) => {
      if chain.error_chain_len(pe) != 2 { return 5; }
      if !chain.error_chain_has(pe, "mid") { return 6; }
    };
    None => { return 7; }
  };
  return 0;
}
