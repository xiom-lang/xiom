// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m234 (XVC-C-08): alignment padding in Vec element strides. The naive field
// sum sized Elem { id: Int; distance: Float32; payload: Vec[Node]; flag: Bool }
// at 52 bytes while LLVM lays it out at 56 (4 bytes of padding before the
// 8-aligned Vec field), so element 1 was pushed/read at a 52-byte stride --
// overlapping elements and a trailing scalar that read uninitialized garbage
// (silent wrong results; probe "BAD detach count", variants V5/V7/V10/V11/V12/
// V14 red). Every Vec[Struct] struct must size with LLVM field alignment.

module m234_vec_elem_padding

use xiom.io;

type Node = {
  id: Int;
}

type Res5 = {
  id: Int;
  distance: Float32;
}

type StoreEntry5 = {
  id: Int;
  payload: Vec[Node];
}

type Store5 = {
  entries: Vec[StoreEntry5];
}

type Elem5 = {
  id: Int;
  distance: Float32;
  payload: Vec[Node];
  flag: Bool;
}

fn store_get5(s: &Store5, id: Int) -> Option[Vec[Node]] {
  var i: Int = 0;
  while i < s.entries.len() {
    if s.entries[i].id == id { return Some(s.entries[i].payload); }
    i = i + 1;
  }
  return None;
}

fn build5(items: &Vec[Res5], store: &Store5, with_payload: Bool) -> Vec[Elem5] {
  var out = Vec[Elem5].new();
  var i: Int = 0;
  while i < items.len() {
    var pl = Vec[Node].new();
    var have: Bool = false;
    if with_payload {
      match store_get5(store, items[i].id) {
        Some(found) => { pl = found; have = true; }
        None => {}
      }
    }
    out.push(Elem5{ id: items[i].id, distance: items[i].distance, payload: pl, flag: have });
    i = i + 1;
  }
  return out;
}

fn bad5() -> Int {
  var store = Store5{ entries: Vec[StoreEntry5].new() };
  var items = Vec[Res5].new();
  items.push(Res5{ id: 1, distance: 0.0 });
  items.push(Res5{ id: 2, distance: 0.5 });
  var off = build5(&items, &store, false);
  var code: Int = 0;
  if off[0].flag { code = code + 1; }
  if off[1].flag { code = code + 2; }
  return code;
}

fn main() -> Int {
  return bad5();
}
