// m319 (REL-20261013-2238-engine-3): engine-shaped mesh model -- `Mesh` is
// the same-named counterpart of stdlib `xiom.geom.geometry_3d.Mesh`.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module xiom.m319mesh.model;

pub type MeshVertex = {
  x: Float64; y: Float64; z: Float64;
  u: Float64; v: Float64; w: Float64;
  nx: Float64; ny: Float64; nz: Float64;
  wx: Float64; wy: Float64; wz: Float64;
}

pub type Mesh = {
  ok: Bool;
  error: Int;
  vertices: Vec[MeshVertex];
  indices: Vec[Int];
}

pub fn mesh_new() -> Mesh {
  return Mesh{ ok: true; error: 0; vertices: Vec[MeshVertex].new(); indices: Vec[Int].new(); };
}

/// Deterministic quad (4 vertices, x = 0..3, w = 1): 4 * 96 = 384 bytes.
pub fn quad_mesh() -> Mesh {
  var m = mesh_new();
  var i = 0;
  while i < 4 {
    m.vertices.push(MeshVertex{
      x: i as Float64; y: 0.0; z: 0.0;
      u: 0.0; v: 0.0; w: 1.0;
      nx: 0.0; ny: 1.0; nz: 0.0;
      wx: 0.0; wy: 0.0; wz: 0.0;
    });
    i = i + 1;
  }
  m.indices.push(0); m.indices.push(1); m.indices.push(2);
  m.indices.push(0); m.indices.push(2); m.indices.push(3);
  return m;
}
