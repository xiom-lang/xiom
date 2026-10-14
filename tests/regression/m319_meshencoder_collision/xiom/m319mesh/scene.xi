// m319 (REL-20261013-2238-engine-3): the imported module side -- builds the
// mesh and calls the encoder (the engine keeps this in the entry module as
// the workaround), forwards an entry-built &Mesh into the encoder, and
// pulls xiom.geom (whose geometry_3d.Mesh collides with the local Mesh).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module xiom.m319mesh.scene;

use xiom.m319mesh.cam;
use xiom.m319mesh.enc;
use xiom.m319mesh.model;

/// Module-side quad builder + encoder call (the engine's broken shape).
pub fn scene_quad_vb() -> Vec[UInt8] {
  var m = quad_mesh();
  return mesh_to_vertex_bytes_f32(&m, true);
}

pub fn scene_quad_ib() -> Vec[UInt8] {
  var m = quad_mesh();
  return mesh_to_index_bytes(&m);
}

/// Entry-built mesh encoded through a module wrapper.
pub fn scene_encode(m: &Mesh) -> Vec[UInt8] {
  return mesh_to_vertex_bytes_f32(m, true);
}

/// Entry-built mesh counted through a module wrapper.
pub fn scene_vertex_count_of(m: &Mesh) -> Int {
  return m.vertices.len();
}

/// Pulls xiom.geom (and geometry_3d.Mesh) into the composition.
pub fn scene_cam() -> Float64 {
  return cam_probe();
}
