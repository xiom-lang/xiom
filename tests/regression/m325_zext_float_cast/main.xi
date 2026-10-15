// m325 (REL-20261014-2123-stdlib): a float-typed pointer element read must
// NOT take an integer extension. Pre-fix `let p = ptr as *Float32; p[i]` in
// an unsafe block emitted `zext float %v to i64` -- invalid IR ("invalid
// cast opcode for cast from 'float' to 'i64'") that rejected the
// p_sweep_single_param clang build inside xiom.simd.vec4.f32x4_load /
// xiom.simd.vec8.f32x8_load (m281's raw-pointer gate made the element width
// dynamic; m322 widened the fallback to scalar pointees). This fixture
// mirrors the single-param sweep shape and checks the VALUES at runtime on a
// real Float32 byte buffer (the pre-fix binary cannot compile it at all).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m325_zext_float_cast;

use xiom.ffi;

type F32x4 = { a: Float32; b: Float32; c: Float32; d: Float32; }

fn load4(ptr: Int) -> F32x4
  requires: ptr != 0
{
  unsafe {
    let p = ptr as *Float32;
    F32x4{ a: p[0]; b: p[1]; c: p[2]; d: p[3]; }
  }
}

fn main() -> Int {
  var buf = Vec[UInt8].new();
  var i: Int = 0;
  while i < 16 { buf.push(0 as UInt8); i = i + 1; }
  let base = buf.as_mut_ptr();
  // Little-endian Float32 bit patterns: 1.5 2.5 3.5 4.5.
  ffi.ptr_write_u32_le(base, 1069547520);
  ffi.ptr_write_u32_le(base + 4, 1075838976);
  ffi.ptr_write_u32_le(base + 8, 1080033280);
  ffi.ptr_write_u32_le(base + 12, 1083179008);
  let v = load4(base as Int);
  if v.a != (1.5 as Float32) { return 1; }
  if v.b != (2.5 as Float32) { return 2; }
  if v.c != (3.5 as Float32) { return 3; }
  if v.d != (4.5 as Float32) { return 4; }
  return 0;
}
