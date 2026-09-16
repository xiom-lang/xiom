// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z21: unsafe+cast+generic+const+contract+match+module+enum+while+compound_assign+derive+Option
const PTR_OFFSET: Int = 2;
type Block = { id: Int; len: Int; } derive[Eq]
fn block_dist[T](b: Block) -> Int {
  return b.len;
}
fn classify[T](b: Block) -> Int {
  if b.len == 0 { return 0; }
  if b.len > 100 { return -1; }
  return 1;
}
module ptrs {
  pub fn dist(b: Block) -> Int { return block_dist(b); }
  pub fn is_live(b: Block) -> Int { return classify(b); }
  pub fn offset_val() -> Int { return PTR_OFFSET; }
  pub fn raw_cast(v: Int) -> *Int { var p: *Int; unsafe { p = v as *Int; } return p; }
}
use ptrs.dist;
use ptrs.is_live;
use ptrs.offset_val;
use ptrs.raw_cast;
fn main() -> Int {
  var blk = Block{ id: 1; len: 8; };
  var d = dist(blk);
  var live = is_live(blk);
  var p = raw_cast(42);
  var chk = 0;
  var i = 0;
  while i < 3 { i += 1; }
  if i == 3 { chk += 1; }
  if live == 1 { chk += 1; }
  if d == 8 { chk += 1; }
  if offset_val() == 2 { chk += 1; }
  if p != unsafe { 0 as *Int } { chk += 1; }
  if chk == 5 { return 0; }
  return 1;
}
