// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m240 (verifier v2, SMT Array memory model): raw pointer store/load
// bodies must emit REAL queries. `requires: base != null` plus the
// pointer-cast null-preservation model prove the X7009 null-safety
// obligations; the per-pointer memory functions model the accesses.
module verify_ptr_mem

pub fn store_word(base: *mut UInt8, word: Int, value: Int)
  requires: base != null
{
  unsafe {
    var p = base as *mut Int;
    *(p + word) = value;
  }
}

pub fn load_word(base: *mut UInt8, word: Int) -> Int
  requires: base != null
{
  unsafe {
    var p = base as *Int;
    return *(p + word);
  }
}
