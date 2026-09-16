// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-S19: Optimization -- inlining simulation (heuristic decision)
type InlineCandidate = { fn_id: Int; body_size: Int; call_count: Int; has_loop: Bool; }
fn make_candidate(id: Int, size: Int, calls: Int, loop: Bool) -> InlineCandidate {
  return InlineCandidate{ fn_id: id; body_size: size; call_count: calls; has_loop: loop; };
}
fn should_inline(c: InlineCandidate) -> Bool {
  if c.has_loop { return false; }
  if c.body_size <= 10 { return true; }
  if c.body_size <= 20 && c.call_count == 1 { return true; }
  return false;
}
fn inline_benefit(c: InlineCandidate) -> Int {
  if !should_inline(c) { return 0; }
  var saved: Int = c.call_count * 3;
  var cost: Int = c.body_size;
  return saved - cost;
}
fn is_recursive(c: InlineCandidate) -> Bool {
  return c.body_size == 0 && c.call_count > 0;
}
fn inline_cost_estimate(c: InlineCandidate) -> Int {
  var base: Int = c.body_size * c.call_count;
  if c.has_loop { base = base * 4; }
  return base;
}
fn main() -> Int {
  var small = make_candidate(1, 5, 10, false);
  var medium = make_candidate(2, 15, 1, false);
  var large = make_candidate(3, 50, 2, false);
  var loop_fn = make_candidate(4, 8, 5, true);
  var edge_small = make_candidate(5, 10, 1, false);
  if !should_inline(small) { return 1; }
  if !should_inline(medium) { return 2; }
  if should_inline(large) { return 3; }
  if should_inline(loop_fn) { return 4; }
  if !should_inline(edge_small) { return 5; }
  if inline_benefit(small) != 25 { return 6; }
  if inline_benefit(large) != 0 { return 7; }
  if is_recursive(medium) { return 8; }
  var ib = inline_cost_estimate(loop_fn);
  if ib != 160 { return 9; }
  return 0;
}
