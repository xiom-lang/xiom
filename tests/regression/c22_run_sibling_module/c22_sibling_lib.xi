// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// C22 lock companion: a sibling module of main.xi, resolvable only when the
// script's own directory reaches the checker's catalog.
module c22_sibling_lib

pub fn bump(n: Int) -> Int {
  return n + 1;
}
