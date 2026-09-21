// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module repro_map_keys

use xiom.collections;

var _coverage: Map[Str, Bool] = Map[Str, Bool].new();

pub fn record(k: Str) {
  _coverage.insert(k, true);
}

pub fn keys_count() -> Int {
  let keys = _coverage.keys();
  return keys.len();
}
