// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m245 (verifier v2 follow-up): the SMT Array memory model must prove
// load-after-store congruence -- a word written through a raw pointer is
// read back by a subsequent dereference of the same address.
module verify_mem_store_load

pub fn roundtrip(base: *mut Int, word: Int, value: Int) -> Int
  requires: base != null
  ensures: result == value
{
  unsafe {
    *(base + word) = value;
    return *(base + word);
  }
}
