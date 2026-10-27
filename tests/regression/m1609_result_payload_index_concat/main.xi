// m1609 (REL-20261010-1609-stdlib): an INLINE Result.value[i] payload
// index in a concat rvalue context must read the element's real type
// (Str), not the erased payload slot's pointer bits. The bound form
// (`let v = ld.value; let s = v[0]`) was already correct; the inline
// form's element resolved through the declared field metadata and
// defaulted to Int, so `"[" + ld.value[0] + "]"` printed a pointer
// number. Workaround was to bind the payload first.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m1609_result_payload_index_concat

use xiom.string;

fn mk_res() -> Result[Vec[Str], Str] {
  var v = Vec[Str].new();
  v.push("aa.txt");
  v.push("bb.txt");
  return Ok(v);
}

fn main() -> Int {
  let ld = mk_res();
  if ld.is_ok != true { return 9; }
  // Control 1: direct comparison of the same inline element is correct.
  if ld.value[0] != "aa.txt" && ld.value[0] != "bb.txt" { return 8; }
  // Case A (bug): inline payload index in a concat rvalue context.
  let shown = "[" + ld.value[0] + "]";
  if !string.str_contains(shown, "aa.txt") && !string.str_contains(shown, "bb.txt") { return 1; }
  // Case B (control, bound form): bind payload + element first.
  let v = ld.value;
  let s = v[0];
  let bound = "[" + s + "]";
  if !string.str_contains(bound, "aa.txt") && !string.str_contains(bound, "bb.txt") { return 2; }
  return 0;
}
