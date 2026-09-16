// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// REPRO R4 (BUG 27 #5): Option[Vec[Str]] payload via unwrap() / var-bound.
// regex/engine.xi regex_captures returns Option[Vec[Str]]: is_some reads
// correctly, the Vec payload is garbage; some shapes fail to compile.
module repro_opt_vec

pub fn captures(s: Str, pat: Str) -> Option[Vec[Str]] {
  if s.len() > 0 {
    var v = Vec[Str].new();
    v.push(s);
    return Some(v);
  }
  return None;
}