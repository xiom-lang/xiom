// m319 (REL-20261013-2238-engine-3): a xiom.geom consumer in the composed
// chain -- loading xiom.geom is what surfaced the same-named Mesh collision.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module xiom.m319mesh.cam;

use xiom.geom;

pub fn cam_probe() -> Float64 {
  var v = vec3_new(3.0, 4.0, 0.0);
  return vec3_length(v);
}
