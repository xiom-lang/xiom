// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m326 (REL-20261014-2123-stdlib part 4): duplicate invariant-helper emission
// for the colliding `LinkedList` leaf, trimmed from p_wave7/p_wave8.
//
// xiom.collections.collections declares the GENERIC `LinkedList[T]` (one Vec
// field + an invariant); xiom.collect.list declares the NON-generic
// `LinkedList` (six fields). The collision triage injects the non-generic
// decl module-qualified while the generic decl keeps the bare `LinkedList`
// key, so:
//   1. the fn-signature injection walk deduped on the QUALIFIED key, pushed
//      the collections decl a SECOND time (same spans), and codegen emitted
//      `LinkedList.invariant_check` twice and clang rejected the module
//      ("invalid redefinition of function 'LinkedList.invariant_check'");
//   2. the bare-key claim typed a user `&mut LinkedList` param as the
//      collections 1-field layout, so the @pre snapshot preclone read the
//      6-field list's head/tail/size as a Vec header -> malloc(huge) NULL ->
//      llvm.trap (0xC000001D) while `ll_len(l)@pre` was live.
//
// Both clause shapes below are the wave7/wave8 reproducers; returns 0 when
// every shape compiles once and holds at runtime.

module m326_dup_invariant_helper

use xiom.collect.list;

fn s_opt_len(l: &LinkedList) -> Option[Int]
  ensures: result.is_some == (ll_len(l) > 0)
{
  return ll_front(l);
}

fn s_pop_len(l: &mut LinkedList) -> Option[Int]
  ensures: result is Some => ll_len(l) == ll_len(l)@pre - 1
  ensures: result is None => ll_len(l) == ll_len(l)@pre
{
  return ll_pop_front(l);
}

fn main() -> Int {
  var l = linked_list_new();
  ll_push_back(&l, 5);
  var f = s_opt_len(&l);
  if !f.is_some { return 1; }
  var e = linked_list_new();
  var fe = s_opt_len(&e);
  if fe.is_some { return 2; }

  ll_push_back(&l, 7);
  var p1 = s_pop_len(&l);
  if !p1.is_some { return 3; }
  if ll_len(&l) != 1 { return 4; }
  var p2 = s_pop_len(&l);
  if !p2.is_some { return 5; }
  var p3 = s_pop_len(&l);
  if p3.is_some { return 6; }
  if ll_len(&l) != 0 { return 7; }
  return 0;
}
