// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m321 (REL-20261013-2119-engine): the witness-AV class pinned at the
// compiler contract level -- an element READ on an EMPTY Vec.
//
// The engine bisect artifacts (bisect_88/bisect_95c) AV'd inside strcmp:
// their check 20 read `events[0]` on an empty Vec[Str] (the tick-5 chord
// can never fire -- its first member was RELEASED before the second
// member's press, so all_down never holds), and the resident heap bytes
// were dereferenced as a Str pointer. The bisect_75 "working" variant
// simply does not contain that check: there is no main-size threshold.
// The same behavior reproduces on the pinned v0.64.4 toolchain, and the
// engine's real probe_input_map battery is 22/22 green -- this is not a
// compiler regression.
//
// Documented contract: default builds keep the historical unchecked read
// (out-of-bounds reads are UB; the witness dereferenced the garbage), and
// --overflow-checks turns the read into a deterministic llvm.trap via the
// read-path bounds guard (expr.rs `bounds_trap`). The e2e lock compiles
// THIS file with --overflow-checks and asserts a NON-ZERO exit; the IR
// twin pins the guard's presence/absence split.
module m321_empty_vec_read

fn main() -> Int {
  var names = Vec[Str].new();
  var first = names[0];
  if first == "never" { return 1; };
  return 0;
}
