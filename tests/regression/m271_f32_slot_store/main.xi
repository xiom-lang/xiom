// m271 (XVC-C-15, REL-20261011-1134-xvector): literal stores into a
// Vec[Float32] slot must NARROW numerically. Pre-fix, the indexed/field
// store lowered the RHS through val_to_i64 and truncated the raw bits:
// an f64 literal stored its low 32 bits (7.0 -> 0.0f) and an Int literal
// stored its raw bits as an f32 pattern (7 -> 9.8e-45). Float32 variables
// and push() convert at the call boundary (controls). The m260 read
// sibling is already fixed; this locks the WRITE side.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m271_f32_slot_store;

use xiom.io;
use xiom.convert;

type VecBox = {
  data: Vec[Float32];
  tag: Int;
}

fn f32str(x: Float32) -> Str {
  return convert.float_to_string(x as Float64);
}

fn main() -> Int {
  var fails: Int = 0;

  // A: Float64 literal into a Vec[Float32] slot (expect 7.0).
  var a = Vec[Float32].new();
  a.push(0.5); a.push(0.5); a.push(0.5);
  a[1] = 7.0;
  if a[1] != 7.0 {
    fails = fails + 1;
    io.println("  [CORRUPT] A f64 literal -> Vec[Float32] slot: a[1]=" + f32str(a[1]));
  }

  // B: Int literal into a Vec[Float32] slot (expect 7.0).
  var b = Vec[Float32].new();
  b.push(0.5); b.push(0.5); b.push(0.5);
  b[1] = 7;
  if b[1] != 7.0 {
    fails = fails + 1;
    io.println("  [CORRUPT] B int literal -> Vec[Float32] slot: b[1]=" + f32str(b[1]));
  }

  // C: Float32 variable into a Vec[Float32] slot (control: expect 7.0).
  var c = Vec[Float32].new();
  c.push(0.5); c.push(0.5); c.push(0.5);
  var cval: Float32 = 7.0;
  c[1] = cval;
  if c[1] != 7.0 {
    fails = fails + 1;
    io.println("  [CORRUPT] C f32 variable -> Vec[Float32] slot: c[1]=" + f32str(c[1]));
  }

  // D: Int literal into a Vec[Float64] slot (expect 7.0 -- same statement,
  // same narrowing semantics for the wider float element).
  var d = Vec[Float64].new();
  d.push(0.5); d.push(0.5); d.push(0.5);
  d[1] = 7;
  if d[1] != 7.0 {
    fails = fails + 1;
    io.println("  [CORRUPT] D int literal -> Vec[Float64] slot: d[1]=" + convert.float_to_string(d[1]));
  }

  // E: Float64 literal into a struct Vec[Float32] field (expect 7.0).
  var edata: Vec[Float32] = Vec[Float32].new();
  edata.push(0.5); edata.push(0.5); edata.push(0.5);
  var e = VecBox { data: edata, tag: 0 };
  e.data[1] = 7.0;
  if e.data[1] != 7.0 {
    fails = fails + 1;
    io.println("  [CORRUPT] E f64 literal -> struct Vec[Float32] field: e.data[1]=" + f32str(e.data[1]));
  }

  // F: push(Float64 literal) (control: expect 7.0).
  var f = Vec[Float32].new();
  f.push(0.5);
  f.push(7.0);
  if f[1] != 7.0 {
    fails = fails + 1;
    io.println("  [CORRUPT] F f64 literal -> push: f[1]=" + f32str(f[1]));
  }

  // G: Float64 literal into a Vec[Float64] slot (control: expect 7.0).
  var g = Vec[Float64].new();
  g.push(0.5);
  g.push(0.5);
  g[1] = 7.0;
  if g[1] != 7.0 {
    fails = fails + 1;
    io.println("  [CORRUPT] G f64 literal -> Vec[Float64] slot: g[1]=" + convert.float_to_string(g[1]));
  }

  io.println("m271_f32_slot_store: " + convert.int_to_string(fails) + " corrupted shapes");
  return fails;
}
