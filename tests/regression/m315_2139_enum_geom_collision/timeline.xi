// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m315 lock: the engine-shaped enum. Variants are deliberately named after
// xiom.geom types (Vec2/Vec3/Vec4). Pre-fix, once ANY co-loaded module pulled
// xiom.geom in, `return Vec3;` here typed as the imported STRUCT Vec3 and the
// return failed; `Vec3{ x; y; z; }` in tl_vec typed as the VARIANT parent
// TrackKind. Both must resolve by context: bare `Vec3` in value position is
// the variant of THIS module's enum; the braced literal is the geom struct.
module timeline;

use xiom.geom;

pub enum TrackKind {
  Scalar,
  Vec2,
  Vec3,
  Vec4,
}

pub fn tl_new() -> TrackKind {
  return Vec3;
}

pub fn tl_code(k: TrackKind) -> Int {
  match k {
    Scalar => { return 0; }
    Vec2 => { return 1; }
    Vec3 => { return 2; }
    Vec4 => { return 3; }
  }
}

pub fn tl_vec(s: Float64) -> Vec3 {
  return Vec3{ x: s; y: s; z: s; };
}
