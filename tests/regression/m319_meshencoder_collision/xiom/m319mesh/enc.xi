// m319 (REL-20261013-2238-engine-3): the engine's byte encoders -- the f32
// slot path (4-byte value + 4-byte zero pad per component, stride 96) and
// the u32 index path (stride 4), both deterministic.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module xiom.m319mesh.enc;

use xiom.m319mesh.model;
use xiom.num.float;
use xiom.serialize.endian;

/// IEEE-754 binary32 bit pattern as a non-negative Int (engine mesh_f32_bits).
fn f32_bits(x: Float64) -> Int {
  var f = x as Float32;
  var d = f as Float64;
  var u = float_bits(d);
  var sign = (u >> 63) & 1;
  var exp64 = (u >> 52) & 0x7FF;
  var frac64 = u & 0xFFFFFFFFFFFFF;
  if exp64 == 0x7FF {
    if frac64 == 0 { return (sign << 31) | 0x7F800000; };
    return (sign << 31) | 0x7FC00000;
  }
  if exp64 == 0 { return sign << 31; };
  var e = exp64 - 1023;
  if e >= 0 - 126 {
    return (sign << 31) | ((e + 127) << 23) | ((frac64 >> 29) & 0x7FFFFF);
  }
  if e < 0 - 149 { return sign << 31; };
  var s = (0 - e) - 97;
  var mant = (0x10000000000000 + frac64) >> s;
  return (sign << 31) | (mant & 0x7FFFFF);
}

fn write_f32_slot(out: &mut Vec[UInt8], x: Float64) {
  write_u32_le(out, f32_bits(x) as UInt32);
  out.push(0); out.push(0); out.push(0); out.push(0);
}

/// GPU (Vulkan) vertex encoding: byte stride 48 (x,y,z,u,v,w) or 96 with
/// `lit` (adds nx,ny,nz,wx,wy,wz), each f32 LE in an 8-byte slot.
pub fn mesh_to_vertex_bytes_f32(m: &Mesh, lit: Bool) -> Vec[UInt8] {
  var out = Vec[UInt8].new();
  var i = 0;
  while i < m.vertices.len() {
    write_f32_slot(&mut out, m.vertices[i].x);
    write_f32_slot(&mut out, m.vertices[i].y);
    write_f32_slot(&mut out, m.vertices[i].z);
    write_f32_slot(&mut out, m.vertices[i].u);
    write_f32_slot(&mut out, m.vertices[i].v);
    write_f32_slot(&mut out, m.vertices[i].w);
    if lit {
      write_f32_slot(&mut out, m.vertices[i].nx);
      write_f32_slot(&mut out, m.vertices[i].ny);
      write_f32_slot(&mut out, m.vertices[i].nz);
      write_f32_slot(&mut out, m.vertices[i].wx);
      write_f32_slot(&mut out, m.vertices[i].wy);
      write_f32_slot(&mut out, m.vertices[i].wz);
    }
    i = i + 1;
  }
  return out;
}

/// Index buffer encoding: u32 LE per index (stride 4).
pub fn mesh_to_index_bytes(m: &Mesh) -> Vec[UInt8] {
  var out = Vec[UInt8].new();
  var i = 0;
  while i < m.indices.len() {
    write_u32_le(&mut out, m.indices[i] as UInt32);
    i = i + 1;
  }
  return out;
}
