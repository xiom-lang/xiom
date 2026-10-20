// m377 -- `ptr.read`/`ptr.write` on a scalar pointee inferred from a cast
// target bound T to the whole pointer ("*Float32") instead of its pointee.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Expected: rc 0, `lane1=2 lane3=4`, and NO `unknown type '*Float32' --
// defaulting to i64` warning. Pre-fix the mono bound T="*Float32": the
// parameter lowered through the unknown-type i64 fallback and the call
// passed an i64* to a float* function, so lane reads returned pointer bits
// (Vec4f.get(1) == 0). The call-site pointer ABI also substituted with the
// stale `param_concrete_types` map instead of this call's type args.
//
// The lane arithmetic here is ELEMENT-based (`data + index`), matching the
// documented `ptr + count` contract. The stdlib Vec4f lane code still uses
// `data + index * 4` (double scaling) and is tracked separately.
//
// Kept as the compiler-side e2e lock for REL-20261019-2249-stdlib-2.

module m377_ptr_scalar_lane

use xiom.io;
use xiom.convert;
use xiom.alloc;
use xiom.ptr;

type Lane4 = { data: *Float32; }

fn lane_read(v: Lane4, index: Int) -> Float32 {
  return ptr.read((v.data + index) as *Float32);
}

fn lane_write(v: Lane4, index: Int, value: Float32) {
  ptr.write((v.data + index) as *Float32, value);
}

fn main() -> Int {
  let raw = alloc.alloc(16);
  var fp: *Float32;
  unsafe { fp = raw as *Float32; }
  let v = Lane4{ data: fp };
  lane_write(v, 0, 1.0);
  lane_write(v, 1, 2.0);
  lane_write(v, 2, 3.0);
  lane_write(v, 3, 4.0);

  let a = lane_read(v, 1);
  let b = lane_read(v, 3);
  io.println("lane1=" + a.to_string() + " lane3=" + b.to_string());

  unsafe { alloc.dealloc(raw, 16); }
  if a != 2.0 { return 1; }
  if b != 4.0 { return 2; }
  return 0;
}
