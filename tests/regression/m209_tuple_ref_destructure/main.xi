// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m209 (packages relay #7): `let (k, v) = &vec[i]` over Vec[(Str, Str)] must
// bind REFERENCES to the tuple components (pre-fix both names got the same
// ptrtoint'd element address), and `k == &key` must content-compare through
// both references. Pre-fix the lookup silently missed (grpc metadata).
fn find(v: &Vec[(Str, Str)], key: Str) -> Bool {
  var i = 0;
  while i < v.len() {
    let (k, val) = &v[i];
    if k == &key { return true; }
    i = i + 1;
  };
  return false;
}

fn main() -> Int {
  var v: Vec[(Str, Str)] = Vec[(Str, Str)].new();
  v.push(("grpc-timeout", "5s"));
  if !find(&v, "grpc-timeout") { return 1; }
  if find(&v, "missing") { return 2; }
  return 0;
}
