// m376 -- multiple mutating generic Map/Set free helpers in one program AV'd.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Expected: rc 0, `m376 ok`. Pre-fix `collections.map_get_or` registered its
// `key: &K` param in the per-body ref-param set, which was never cleared
// between monomorphisations; the later `map_insert_if_absent`'s by-value
// `key` then compiled as a reference at `m.insert(key, value)` (inttoptr +
// load of the VALUE) and dereferenced address 2 -> 0xC0000005. The Set pair
// reproduced the same leak. Singles and read-only combinations were clean.
//
// Kept as the compiler-side e2e lock for REL-20261019-2249-stdlib.

module m376_map_set_helpers

use xiom.io;
use xiom.collections;

fn main() -> Int {
  var m = Map[Int, Int].new();
  let k = 1;
  let missing = map_get_or(&m, &k, 42);
  let inserted = map_insert_if_absent(&mut m, 2, 22);
  let present = map_get_or(&m, &2, -1);
  if missing != 42 { return 1; }
  if !inserted { return 2; }
  if present != 22 { return 3; }
  if m.len() != 1 { return 4; }

  var s = Set[Int].new();
  let absent = set_contains(&s, &1);
  set_insert(&mut s, 2);
  let found = set_contains(&s, &2);
  if absent { return 5; }
  if !found { return 6; }
  if s.len() != 1 { return 7; }

  io.println("m376 ok");
  return 0;
}
