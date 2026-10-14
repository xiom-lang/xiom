// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m315 lock: the consumer module (the engine camera-player combination)
// loads BOTH the enum's module and xiom.geom. Its `CamState` fields are
// geom `Vec3` structs; pre-fix the struct literals in `cam_new` were typed
// as the enum variant's parent (TrackKind) and the compile aborted with
// T001 field type mismatch.
module camera;

use xiom.geom;
use timeline;

pub type CamState = {
  pos: Vec3;
  dir: Vec3;
  kind: TrackKind;
}

pub fn cam_new(k: TrackKind) -> CamState {
  return CamState{
    pos: Vec3{ x: 1.0; y: 2.0; z: 3.0; };
    dir: Vec3{ x: 0.0; y: 0.0; z: 1.0; };
    kind: k
  };
}

pub fn cam_len(c: CamState) -> Float64 {
  let l = geom.vec3_length(c.pos);
  return l;
}
