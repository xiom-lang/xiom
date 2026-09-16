// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// m39_round9_set_abi -- round-9 (2026-08-20) regression:
// Set container ABI mismatch -- the compiler had NO builtin Set layout
// (unlike Vec/Slice/Map), so every Set value erased to i64 while the
// stdlib's methods operated on %struct.Set. `Set[Int].new()` hijacked
// Reverse.new, Set-typed params/returns/fields compiled as i64, and
// `holder.s.insert(10)` mono'd Vec.insert with a %struct.Vec* receiver.
// Fixes: inject the stdlib Set type (drop "Set" from checker PRIMITIVES),
// is_container_vec_field accepts only Vec/Slice/Array fields, field
// receivers resolve generic-arg types ("Set[Int]" -> "Set") for the fn_key,
// mono pointer-self passes the FIELD ADDRESS for field receivers, and the
// pointer-len handler skips struct pointees.
module m39_round9_set_abi
use xiom.collections;

type Holder = { s: Set[Int]; }

fn set_len(s: &Set[Int]) -> Int {
  return s.len();
}

fn make_set() -> Set[Int] {
  var s = Set[Int].new();
  s.insert(7);
  s.insert(8);
  return s;
}

fn main() -> Int {
  // 1. Set[Str] (pointer elements).
  var ss = Set[Str].new();
  ss.insert("a");
  ss.insert("b");
  if ss.len() != 2 { return 1; }
  if !ss.contains("a") { return 2; }
  // 2. Set passed by reference.
  var si = Set[Int].new();
  si.insert(3);
  if set_len(&si) != 1 { return 3; }
  // 3. Set returned from a fn.
  var s2 = make_set();
  if s2.len() != 2 { return 4; }
  if !s2.contains(7) { return 5; }
  // 4. Set in a struct field (field-receiver &mut self).
  var holder = Holder{ s: Set[Int].new() };
  holder.s.insert(10);
  if holder.s.len() != 1 { return 6; }
  // 5. Set union/intersection (mono'd with &Set params).
  var s3 = Set[Int].new();
  s3.insert(7);
  s3.insert(9);
  var u = s2.union(&s3);
  if u.len() != 3 { return 7; }
  if !u.contains(9) { return 8; }
  return 0;
}
