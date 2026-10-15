// m335 (REL-20261014-1208-bindings-2, B-29 triage): the reported "big-int
// bitops are wrong for values >= 2^31" class is arithmetically mis-grounded.
// The bundle probe's literal 3132620800 is 0xBAB80000 -- NOT 0xBAD00000
// (0xBAD00000 = 3134193664). Its "expected" values (shift20=2989,
// and_mask=3132620800, div20=2989) are impossible for that literal under any
// shift/and/div semantics, while the values it reports as wrong
// (shift20=2987, and_mask=3132096512=0xBAB00000, div20=2987) are exactly the
// correct results for it. Re-verified on the official v0.64.5 archive AND on
// main: no 32-bit truncation path exists -- the emitter lowers Int ops as
// i64 (`ashr i64` / `and i64` / `sdiv i64`) and the true NGX constant works
// in every shape. This lock pins the correct semantics for the whole class
// so the false positive cannot flip into a real regression:
//   [1] the probe's literal as written -> 0xBAB80000-shaped results;
//   [2] the true 0xBAD00000 = 3134193664 (decimal + hex, folded + runtime);
//   [3] the upstream SUCCEED/FAILED macro shape over 0xBAD00000|N, N in
//       0..18, in both the bitwise and the boundary-arithmetic forms;
//   [4] 32-bit provenance conversions (UInt32->Int zext, Int32->Int sext)
//       through the masked compare;
//   [5] a runtime shift sweep (v >> s == v / (1 << s));
//   [6] the bundle probe's own sub-2^31 controls.
// NOTE for the bindings lane: the shipped xiom.dlss constant
// NVSDK_NGX_RESULT_FAIL = 3132620800 (comment "0xBAD00000") is 0xBAB80000;
// the real code is 3134193664. See docs/COMPILER_BUGS.md (m335 entry).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m335_bigint_bitops;

use xiom.io;
use xiom.convert;

// Runtime (non-foldable) sources for the big values.
var g_bad: Int = 3134193664;      // 0xBAD00000
var g_probe: Int = 3132620800;    // 0xBAB80000 (the bundle probe's literal)
var g_shift: Int = 20;
var g_mask: Int = 4293918720;     // 0xFFF00000
var g_div: Int = 1048576;         // 2^20

// Upstream NVSDK_NGX_SUCCEED/FAILED shape.
fn ngx_failed(rc: Int) -> Bool {
  return (rc & 0xFFF00000) == 0xBAD00000;
}

// Upstream failure-code construction: 0xBAD00000 | N.
fn ngx_code(n: Int) -> Int {
  return 0xBAD00000 | n;
}

fn u32_roundtrip() -> Int {
  let u: UInt32 = 0xBAD00000;
  return u as Int;
}

fn i32_roundtrip() -> Int {
  let s: Int32 = 0 - 1160773632;   // bit pattern 0xBAD00000
  return s as Int;
}

pub fn main() -> Int {
  var fails: Int = 0;

  // [1] the bundle probe's literal, as written (3132620800 = 0xBAB80000):
  // the correct results are exactly the ones the probe called "wrong".
  let p_sh = 3132620800 >> 20;
  let p_and = 3132620800 & 4293918720;
  let p_div = 3132620800 / 1048576;
  let p_mod = 3132620800 % 1048576;
  if p_sh != 2987 {
    fails = fails + 1;
    io.println("  [1] 3132620800 >> 20 wrong: " + convert.int_to_string(p_sh));
  }
  if p_and != 3132096512 {   // 0xBAB00000
    fails = fails + 1;
    io.println("  [1] 3132620800 & 0xFFF00000 wrong: " + convert.int_to_string(p_and));
  }
  if p_div != 2987 {
    fails = fails + 1;
    io.println("  [1] 3132620800 / 2^20 wrong: " + convert.int_to_string(p_div));
  }
  if p_mod != 524288 {       // 0x80000
    fails = fails + 1;
    io.println("  [1] 3132620800 % 2^20 wrong: " + convert.int_to_string(p_mod));
  }

  // [2] the true constant 0xBAD00000 = 3134193664: constant-folded, hex,
  // and runtime (module globals defeat any fold).
  let t_sh = 3134193664 >> 20;
  let t_and = 3134193664 & 4293918720;
  let t_div = 3134193664 / 1048576;
  if t_sh != 2989 {
    fails = fails + 1;
    io.println("  [2] 3134193664 >> 20 wrong: " + convert.int_to_string(t_sh));
  }
  if t_and != 3134193664 {
    fails = fails + 1;
    io.println("  [2] 3134193664 & 0xFFF00000 wrong: " + convert.int_to_string(t_and));
  }
  if t_div != 2989 {
    fails = fails + 1;
    io.println("  [2] 3134193664 / 2^20 wrong: " + convert.int_to_string(t_div));
  }

  let h_sh = 0xBAD00000 >> 20;
  let h_and = 0xBAD00000 & 0xFFF00000;
  let h_div = 0xBAD00000 / 0x100000;
  if h_sh != 2989 {
    fails = fails + 1;
    io.println("  [2] 0xBAD00000 >> 20 wrong: " + convert.int_to_string(h_sh));
  }
  if h_and != 0xBAD00000 {
    fails = fails + 1;
    io.println("  [2] 0xBAD00000 & 0xFFF00000 wrong: " + convert.int_to_string(h_and));
  }
  if h_div != 2989 {
    fails = fails + 1;
    io.println("  [2] 0xBAD00000 / 0x100000 wrong: " + convert.int_to_string(h_div));
  }

  let r_sh = g_bad >> g_shift;
  let r_and = g_bad & g_mask;
  let r_div = g_bad / g_div;
  let r_or = (g_bad | 12) & g_mask;
  if r_sh != 2989 {
    fails = fails + 1;
    io.println("  [2] runtime >> wrong: " + convert.int_to_string(r_sh));
  }
  if r_and != 3134193664 {
    fails = fails + 1;
    io.println("  [2] runtime & wrong: " + convert.int_to_string(r_and));
  }
  if r_div != 2989 {
    fails = fails + 1;
    io.println("  [2] runtime / wrong: " + convert.int_to_string(r_div));
  }
  if r_or != 3134193664 {
    fails = fails + 1;
    io.println("  [2] runtime (|12) & mask wrong: " + convert.int_to_string(r_or));
  }

  // [3] every pinned failure code classifies as failed through the upstream
  // bitwise macro, matches the true decimal constant, and agrees with the
  // boundary-arithmetic form over the same correct constant.
  var n = 0;
  while n <= 18 {
    let rc = ngx_code(n);
    if !ngx_failed(rc) {
      fails = fails + 1;
      io.println("  [3] ngx_failed missed code " + convert.int_to_string(n));
    }
    if rc != 3134193664 + n {
      fails = fails + 1;
      io.println("  [3] 0xBAD00000|N wrong for N=" + convert.int_to_string(n));
    }
    let d = rc - 3134193664;
    if d < 0 || d > 18 {
      fails = fails + 1;
      io.println("  [3] boundary form missed code " + convert.int_to_string(n));
    }
    n = n + 1;
  }
  if ngx_failed(1) {
    fails = fails + 1;
    io.println("  [3] success code 1 classified as failed");
  }
  if ngx_failed(0) {
    fails = fails + 1;
    io.println("  [3] code 0 classified as failed");
  }

  // [4] 32-bit provenance: UInt32 -> Int is a zero-extend (0xBAD00000 stays
  // 3134193664), Int32 -> Int sign-extends the 0xBAD00000 bit pattern, and
  // the masked compare still sees the low 32 bits.
  let u = u32_roundtrip();
  if u != 3134193664 {
    fails = fails + 1;
    io.println("  [4] UInt32->Int wrong: " + convert.int_to_string(u));
  }
  if (u & 4293918720) != 3134193664 {
    fails = fails + 1;
    io.println("  [4] UInt32->Int masked wrong: " + convert.int_to_string(u & 4293918720));
  }
  let s = i32_roundtrip();
  if s != 0 - 1160773632 {
    fails = fails + 1;
    io.println("  [4] Int32->Int wrong: " + convert.int_to_string(s));
  }
  if (s & 4293918720) != 3134193664 {
    fails = fails + 1;
    io.println("  [4] Int32->Int masked wrong: " + convert.int_to_string(s & 4293918720));
  }
  if !ngx_failed(s) {
    fails = fails + 1;
    io.println("  [4] Int32-provenance rc missed by ngx_failed");
  }

  // [5] runtime shift sweep on a big value: v >> s == v / (1 << s).
  var v = 3134193664;
  var sh = 0;
  while sh <= 40 {
    if (v >> sh) != (v / (1 << sh)) {
      fails = fails + 1;
      io.println("  [5] shift sweep mismatch at s=" + convert.int_to_string(sh));
    }
    sh = sh + 1;
  }

  // [6] the bundle probe's own sub-2^31 controls (must stay faithful).
  if (2147483648 >> 20) != 2048 {
    fails = fails + 1;
    io.println("  [6] control 2147483648 >> 20 wrong");
  }
  if (3000000000 >> 20) != 2861 {
    fails = fails + 1;
    io.println("  [6] control 3000000000 >> 20 wrong");
  }
  if (3132620800 - 3132620800) != 0 {
    fails = fails + 1;
    io.println("  [6] control 3132620800 - 3132620800 wrong");
  }
  if (4294967295 & 4293918720) != 4293918720 {
    fails = fails + 1;
    io.println("  [6] control 0xFFFFFFFF & 0xFFF00000 wrong");
  }
  if (4294967295 >> 24) != 255 {
    fails = fails + 1;
    io.println("  [6] control 0xFFFFFFFF >> 24 wrong");
  }
  if ((0 - 1) >> 1) != (0 - 1) {
    fails = fails + 1;
    io.println("  [6] control (-1) >> 1 wrong");
  }
  if (4294967296 >> 32) != 1 {
    fails = fails + 1;
    io.println("  [6] control 2^32 >> 32 wrong");
  }

  io.println("m335_bigint_bitops: " + convert.int_to_string(fails) + " failing checks");
  return fails;
}
