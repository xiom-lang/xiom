// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m313 (REL-20261013-1445 compiler half): `ffi.free` call sites must lower
// through the stdlib `ffi.free` wrapper (arena-aware since stdlib wave 124:
// `xiom_guard_free` no-ops on guard-arena pointers, libc-frees main-heap
// pointers). Pre-fix the codegen `free` builtin hijacked every `free` leaf
// call and emitted bare libc `@free`; inside a confined block the matching
// `ffi.alloc` sits in the guard arena (the compiler routes confined alloc to
// @xiom_guard_alloc), so libc free corrupted/spun the process --
// 1548-bindings-5 alloc-guard-spin (VEH retry loop at 100% CPU, no output).
//
// Part A (outside confinement): a MAIN-HEAP alloc/free roundtrip must still
// free correctly (no leak crash) -- both pins.
// Part B (inside one confined block): alloc + write + read + free must pair
// on the same arena and complete (pre-fix: spin, no output).
module m313_alloc_guard_free

use xiom.ffi;
use xiom.io;
use xiom.convert;

fn main() -> Int
  requires: true
{
  let q = ffi.alloc(16);
  ffi.ptr_write_u64_le(q, 9);
  let m = ffi.ptr_read_u64_le(q);
  ffi.free(q);
  io.println("m=" + convert.int_to_string(m));
  io.flush_stdout();
  unsafe {
    let p = ffi.alloc(8);
    ffi.ptr_write_u64_le(p, 7);
    let v = ffi.ptr_read_u64_le(p);
    ffi.free(p);
    io.println("v=" + convert.int_to_string(v));
    io.flush_stdout();
    return 0
  }
}
