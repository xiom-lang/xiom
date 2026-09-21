// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_payload_ref
// BUG 25 #10 (crypto): a match-bound Vec payload from a catalog
// Option/Result was bound as an i64 heap HANDLE; `&v` passed the address of
// the HANDLE SLOT to a `&Vec[T]` param, which read garbage (len=1 vs 2) and
// crashed crypto's aes_decrypt(&key, &ciphertext). Payloads must bind as a
// real %struct.Vec so &v, by-value passing, and methods all work.

use m37_catmod;

fn _take(v: &Vec[Int]) -> Int {
  return v.len();
}

fn _take2(v: Vec[Int]) -> Int {
  return v.len();
}

fn main() -> Int {
  var o = m37_catmod.mk_optvec();
  var bound = 0;
  var via_ref = 0;
  var via_val = 0;
  match o {
    Some(v) => {
      bound = v.len();
      via_ref = _take(&v);
      via_val = _take2(v);
    }
    _ => {}
  }
  if bound != 2 { return 1; }
  if via_ref != 2 { return 2; }
  if via_val != 2 { return 3; }
  return 0;
}
