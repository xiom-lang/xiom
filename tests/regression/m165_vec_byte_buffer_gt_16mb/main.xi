// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m165 lock: a Vec[UInt8] byte buffer must grow past the old 2^24-element
// ceiling (16 MB). A >16 MB buffer (e.g. a 20 MB file) trapped via
// llvm.trap() in the growth path; the ceiling is now 2^32 elements so the
// realloc null-check is the real OOM trap.
module m165_vec_byte_buffer_gt_16mb

fn main() -> Int {
  var v = Vec[UInt8].new();
  var i = 0;
  while i < 16777217 {
    v.push(0u8);
    i = i + 1;
  }
  if v.len() != 16777217 { return 1; }
  if v[16777216] != 0u8 { return 2; }
  v.push(7u8);
  if v[16777216] != 0u8 { return 3; }
  if v[16777217] != 7u8 { return 4; }
  return 0;
}
