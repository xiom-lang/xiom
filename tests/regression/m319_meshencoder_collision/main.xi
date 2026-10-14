// m319 (REL-20261013-2238-engine-3, meshencoder): a mesh-encoder call from
// an IMPORTED module returned a corrupted buffer once the CLI chain pulled
// xiom.geom in -- the engine's `Mesh` (xiom.engine.render.mesh) collided
// with the stdlib `xiom.geom.geometry_3d.Mesh` ({vertices: Vec[Point3];
// indices: Vec[Int]}), so one side compiled against the wrong layout and a
// read through `m.vertices` hit garbage (the engine: 9216 bytes for a
// 384-byte quad + AV; workaround: builders in the entry module). Fixed by
// the same-named-type module identity work (m278); this fixture pins the
// composition: entry-built mesh through a module wrapper, module-built mesh
// through the encoder, and the xiom.geom pull -- with byte-exact compares.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m319_meshencoder_collision.main;

use xiom.io;
use xiom.convert;
use xiom.m319mesh.enc;
use xiom.m319mesh.model;
use xiom.m319mesh.scene;

fn diff_count(a: &Vec[UInt8], b: &Vec[UInt8]) -> Int {
  var n = a.len();
  if b.len() < n { n = b.len(); }
  var d = 0;
  var i = 0;
  while i < n {
    if a[i] != b[i] { d = d + 1; }
    i = i + 1;
  }
  return d;
}

/// The same deterministic quad built in the ENTRY module (the workaround
/// shape: builders stay local) -- byte-identical to model.quad_mesh.
fn entry_quad_vb() -> Vec[UInt8] {
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
  return mesh_to_vertex_bytes_f32(&m, true);
}

/// devcli's fixture_quad_ib (the workaround shape: builder in the entry).
fn entry_quad_ib() -> Vec[UInt8] {
  var qidx = Vec[Int].new();
  qidx.push(0);
  qidx.push(1);
  qidx.push(2);
  qidx.push(0);
  qidx.push(2);
  qidx.push(3);
  var qim = Mesh{ ok: true; error: 0; vertices: Vec[MeshVertex].new(); indices: qidx };
  return mesh_to_index_bytes(&qim);
}

fn local_quad() -> Mesh {
  var qv = Vec[MeshVertex].new();
  qv.push(MeshVertex{ x: 0.0 - 0.5; y: 0.5; z: 0.5; u: 0.0; v: 0.0; w: 1.0; nx: 0.0; ny: 0.0; nz: 1.0; wx: 0.0; wy: 0.0; wz: 0.0; });
  qv.push(MeshVertex{ x: 0.5; y: 0.5; z: 0.5; u: 1.0; v: 0.0; w: 1.0; nx: 0.0; ny: 0.0; nz: 1.0; wx: 0.0; wy: 0.0; wz: 0.0; });
  return Mesh{ ok: true; error: 0; vertices: qv; indices: Vec[Int].new() };
}

fn main() -> Int {
  var fails: Int = 0;

  var va = entry_quad_vb();
  var ia = entry_quad_ib();
  if va.len() != 384 { fails = fails + 1; }
  if ia.len() != 24 { fails = fails + 1; }

  // The CLI-chain shape: xiom.geom (and its same-named Mesh) is pulled in.
  var cd = scene_cam();
  if cd != 5.0 { fails = fails + 1; }

  // Imported module builds the mesh and calls the encoder.
  var vb = scene.scene_quad_vb();
  var ib = scene.scene_quad_ib();
  if vb.len() != 384 { fails = fails + 1; }
  if ib.len() != 24 { fails = fails + 1; }
  if diff_count(&va, &vb) != 0 { fails = fails + 1; }
  if diff_count(&ia, &ib) != 0 { fails = fails + 1; }

  // Entry-built mesh encoded through a module-side wrapper (the leg that
  // still AVed through m277 on the v0.64.4 pin).
  var m = local_quad();
  var vc = scene.scene_encode(&m);
  if vc.len() != 192 { fails = fails + 1; }
  var n = scene.scene_vertex_count_of(&m);
  if n != 2 { fails = fails + 1; }

  io.println("m319_meshencoder_collision: fails=" + int_to_string(fails));
  return fails;
}
