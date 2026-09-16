// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use repro_tuple_destructure;

fn main() -> Int {
  // THE AES-GCM SHAPE: let (vec, int) = fn()
  let (v, nr) = key_expansion();
  var r = use_key(&v, nr);
  if r != 0 { return r; }
  return 0;
}
