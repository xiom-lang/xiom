// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M59: guard-arena escape via outer-Vec growth inside a confined unsafe
// block -- stdlib report R4 (blocks the CSPRNG flip). A Vec created OUTSIDE
// an unsafe block whose capacity growth happens INSIDE was migrated into the
// guard arena by xiom_guard_realloc; the arena is discarded wholesale at
// block exit, so the returned Vec's data pointer dangled: byte reads AV'd
// (0xC0000005) -- reliably once growth passed the initial 16-byte capacity
// (the 4096-byte chunk loop in crypto.os_secure_random_bytes). Fixed in
// xiom_runtime.c: main-heap pointers now grow with plain realloc (arena
// membership check); only arena-born blocks keep the arena path.
module m59_guard_arena_escape

use xiom.crypto;
use xiom.io;
use xiom.convert;

fn main() -> Int {
  // 5000 bytes forces the multi-chunk loop: capacity growth well past the
  // initial 16-byte Vec buffer, all inside the confined block.
  var a = xiom.crypto.os_secure_random_bytes(5000);
  var b = xiom.crypto.os_secure_random_bytes(5000);
  if a.len() != 5000 { return 1; }
  if b.len() != 5000 { return 2; }
  var s = 0;
  var i = 0;
  while i < 5000 {
    s = s + (a[i] as Int);
    i = i + 1;
  }
  i = 0;
  while i < 5000 {
    s = s + (b[i] as Int);
    i = i + 1;
  }
  // 10000 uniform bytes in [0,255]: sum can never legitimately hit the
  // endpoints (probability ~2^-40000 each). Catches all-zero/all-255 data.
  if s == 0 { return 3; }
  if s == 2550000 { return 4; }
  return 0;
}
