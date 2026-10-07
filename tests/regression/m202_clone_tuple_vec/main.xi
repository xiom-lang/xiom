// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m202 (packages grpc tuple-vec relay): `("k".clone(), "v".clone())` in a
// Vec[(Str, Str)] push must keep the tuple element types (Str/Str). Pre-fix
// the tuple-element inference bound the declared `.clone` suffix (the derived
// MaybeUninit.clone), so the literal was built as
// Tuple__MaybeUninit__MaybeUninit (32-byte elements) against the Vec header's
// size and reading metadata[0].0 crashed 0xC0000005 / returned garbage.
type Req = {
  metadata: Vec[(Str, Str)];
}

fn set(req: &mut Req, key: Str, value: Str) {
  var found = false;
  var i = 0;
  while i < req.metadata.len() {
    let (k, v) = &req.metadata[i];
    if k == &key {
      req.metadata[i] = (key.clone(), value.clone());
      found = true;
    };
    i = i + 1;
  };
  if !found {
    req.metadata.push((key.clone(), value.clone()));
  };
}

fn main() -> Int {
  // Local Vec push + tuple read.
  var meta: Vec[(Str, Str)] = Vec[(Str, Str)].new();
  meta.push(("k".clone(), "v".clone()));
  if meta.len() != 1 { return 2; }
  if !(meta[0].0 == "k") { return 1; }
  if !(meta[0].1 == "v") { return 3; }

  // Struct-field Vec mutated by a &mut helper, then read (the grpc shape).
  var req = Req{ metadata: Vec[(Str, Str)].new() };
  set(&mut req, "grpc-timeout", "5s");
  if req.metadata.len() != 1 { return 5; }
  if !(req.metadata[0].0 == "grpc-timeout") { return 4; }
  if !(req.metadata[0].1 == "5s") { return 6; }
  return 0;
}
