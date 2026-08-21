module m37_simd_runtime
// Compiler SIMD/ISA flags (-mavx -mavx2 -mavx512f/bw/dq/vl on native x86_64)
// + stdlib/runtime/simd_runtime.c end-to-end. Exercises the SSE ops (always
// available on x86-64) and the AVX ops (attribute-compiled, runtime-dispatched
// via xiom_simd_has_avx2 -- the production pattern: wide-ISA code only runs
// when CPUID says the CPU supports it). Also sanity-checks the detection
// bitmask. The AVX-512 global flag exists so new runtime C can use _mm512
// intrinsics without per-function attributes -- execution must still be gated
// by xiom_simd_has_avx512().

extern "C" {
  fn xiom_simd_available() -> Int32;
  fn xiom_simd_has_sse2() -> Int32;
  fn xiom_simd_has_avx() -> Int32;
  fn xiom_simd_has_avx2() -> Int32;
  fn xiom_simd_has_avx512() -> Int32;
  fn xiom_simd_f32x4_add(a: *UInt8, b: *UInt8, out: *UInt8);
  fn xiom_simd_f32x4_sub(a: *UInt8, b: *UInt8, out: *UInt8);
  fn xiom_simd_f32x4_mul(a: *UInt8, b: *UInt8, out: *UInt8);
  fn xiom_simd_f32x4_div(a: *UInt8, b: *UInt8, out: *UInt8);
  fn xiom_simd_f32x4_sqrt(a: *UInt8, out: *UInt8);
  fn xiom_simd_f32x4_min(a: *UInt8, b: *UInt8, out: *UInt8);
  fn xiom_simd_f32x4_max(a: *UInt8, b: *UInt8, out: *UInt8);
  fn xiom_simd_f32x4_dot(a: *UInt8, b: *UInt8) -> Float32;
  fn xiom_simd_i32x4_add(a: *UInt8, b: *UInt8, out: *UInt8);
  fn xiom_simd_i32x4_sub(a: *UInt8, b: *UInt8, out: *UInt8);
  fn xiom_simd_i32x4_mul(a: *UInt8, b: *UInt8, out: *UInt8);
  fn xiom_simd_f32x8_add(a: *UInt8, b: *UInt8, out: *UInt8);
  fn xiom_simd_f32x8_mul(a: *UInt8, b: *UInt8, out: *UInt8);
  fn xiom_simd_f64x4_add(a: *UInt8, b: *UInt8, out: *UInt8);
  fn xiom_simd_f64x4_mul(a: *UInt8, b: *UInt8, out: *UInt8);
}

fn main() -> Int {
  var a = Vec[Float32].new();
  var b = Vec[Float32].new();
  var o = Vec[Float32].new();
  a.push(1.0); a.push(2.0); a.push(3.0); a.push(4.0);
  b.push(10.0); b.push(20.0); b.push(30.0); b.push(40.0);
  o.push(0.0); o.push(0.0); o.push(0.0); o.push(0.0);

  unsafe {
    // SSE: add/sub/mul -- exact in f32 for integer operands
    xiom_simd_f32x4_add(a.data, b.data, o.data);
    if o[0] != 11.0 || o[3] != 44.0 { return 1; }
    xiom_simd_f32x4_sub(b.data, a.data, o.data);
    if o[0] != 9.0 || o[2] != 27.0 { return 2; }
    xiom_simd_f32x4_mul(a.data, b.data, o.data);
    if o[1] != 40.0 || o[3] != 160.0 { return 3; }
    // SSE: div with exact binary results (1/4, 2/4, 3/4, 4/4)
    var c = Vec[Float32].new();
    c.push(4.0); c.push(4.0); c.push(4.0); c.push(4.0);
    xiom_simd_f32x4_div(a.data, c.data, o.data);
    if o[0] != 0.25 || o[1] != 0.5 || o[2] != 0.75 || o[3] != 1.0 { return 4; }
    // SSE: sqrt of perfect squares
    var s = Vec[Float32].new();
    s.push(4.0); s.push(9.0); s.push(16.0); s.push(25.0);
    xiom_simd_f32x4_sqrt(s.data, o.data);
    if o[0] != 2.0 || o[1] != 3.0 || o[2] != 4.0 || o[3] != 5.0 { return 5; }
    // SSE: min/max
    xiom_simd_f32x4_min(a.data, b.data, o.data);
    if o[0] != 1.0 || o[3] != 4.0 { return 6; }
    xiom_simd_f32x4_max(a.data, b.data, o.data);
    if o[0] != 10.0 || o[3] != 40.0 { return 7; }
    // SSE4.1: dot product 1*10+2*20+3*30+4*40 = 300
    var d = xiom_simd_f32x4_dot(a.data, b.data);
    if d != 300.0 { return 8; }
  }

  // SSE2 integer ops (Vec[Int32] stores 4-byte elements -- matches int*)
  var ia = Vec[Int32].new();
  var ib = Vec[Int32].new();
  var io = Vec[Int32].new();
  ia.push(1); ia.push(2); ia.push(3); ia.push(4);
  ib.push(10); ib.push(20); ib.push(30); ib.push(40);
  io.push(0); io.push(0); io.push(0); io.push(0);
  unsafe {
    xiom_simd_i32x4_add(ia.data, ib.data, io.data);
    if io[0] != 11 || io[3] != 44 { return 9; }
    xiom_simd_i32x4_sub(ib.data, ia.data, io.data);
    if io[1] != 18 || io[3] != 36 { return 10; }
    // SSE4.1: i32 multiply 2*20=40, 4*40=160
    xiom_simd_i32x4_mul(ia.data, ib.data, io.data);
    if io[1] != 40 || io[3] != 160 { return 11; }
  }

  // Detection bitmask sanity: SSE2 always set on x86-64; AVX2 implies AVX
  var avail: Int32 = 0;
  var has_sse2: Int32 = 0;
  var has_avx: Int32 = 0;
  var has_avx2: Int32 = 0;
  var has_avx512: Int32 = 0;
  unsafe {
    avail = xiom_simd_available();
    has_sse2 = xiom_simd_has_sse2();
    has_avx = xiom_simd_has_avx();
    has_avx2 = xiom_simd_has_avx2();
    has_avx512 = xiom_simd_has_avx512();
  }
  if avail == 0 { return 12; }
  if has_sse2 != 1 { return 13; }
  if has_avx2 == 1 && has_avx != 1 { return 14; }
  if has_avx512 == 1 && has_avx2 != 1 { return 15; }

  // AVX ops: attribute-compiled, dispatch-gated (production pattern) --
  // verify results only when the CPU supports AVX2; on older CPUs the
  // dispatch gate keeps the wide-ISA call off the execution path.
  if has_avx2 == 1 {
    var a8 = Vec[Float32].new();
    var b8 = Vec[Float32].new();
    var o8 = Vec[Float32].new();
    var i: Int = 0;
    while i < 8 { a8.push(1.0 + i as Float64); b8.push(10.0 + i as Float64); o8.push(0.0); i = i + 1; }
    unsafe {
      xiom_simd_f32x8_add(a8.data, b8.data, o8.data);
      if o8[0] != 11.0 || o8[7] != 25.0 { return 16; }
      xiom_simd_f32x8_mul(a8.data, b8.data, o8.data);
      if o8[0] != 10.0 || o8[7] != 136.0 { return 17; }
    }
    // f64x4 ops through Vec[Float64]
    var fa = Vec[Float64].new();
    var fb = Vec[Float64].new();
    var fo = Vec[Float64].new();
    fa.push(1.0); fa.push(2.0); fa.push(3.0); fa.push(4.0);
    fb.push(10.0); fb.push(20.0); fb.push(30.0); fb.push(40.0);
    fo.push(0.0); fo.push(0.0); fo.push(0.0); fo.push(0.0);
    unsafe {
      xiom_simd_f64x4_add(fa.data, fb.data, fo.data);
      if fo[0] != 11.0 || fo[3] != 44.0 { return 18; }
      xiom_simd_f64x4_mul(fa.data, fb.data, fo.data);
      if fo[1] != 40.0 || fo[3] != 160.0 { return 19; }
    }
  }

  return 0;
}
