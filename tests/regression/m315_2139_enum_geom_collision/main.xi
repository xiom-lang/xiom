// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m315 lock (REL-20261013-2139-engine): an enum whose variant NAMES collide
// with xiom.geom type names (Vec2/Vec3/Vec4 -- the engine's TrackKind shape,
// renamed to Float2/Float3/Float4 to work around this). The enum's module
// compiles fine alone; ANY module that loads both it and xiom.geom failed:
// the checker typed the geom struct literal `Vec3{...}` as the enum variant's
// parent (T001 "field 'pos' type mismatch: expected Vec3, found TrackKind"),
// and inside the enum's own module a bare `return Vec3;` bound the imported
// struct instead of the variant. Pre-fix the entry compile fails rc 1;
// post-fix the whole combination compiles and the values below are exact.
module m315_2139_enum_geom_collision;

use xiom.io;
use xiom.convert;
use xiom.geom;
use timeline;
use camera;

fn main() -> Int {
  // The enum's module + geom loaded together in the ENTRY module.
  let v = Vec3{ x: 9.0; y: 8.0; z: 7.0 };
  if v.z != 7.0 {
    io.println("m315: [CORRUPT] bare Vec3 literal z=" + convert.float_to_string(v.z));
    return 4;
  }

  let k = timeline.tl_new();
  let kc = timeline.tl_code(k);
  if kc != 2 {
    io.println("m315: [CORRUPT] variant code=" + convert.int_to_string(kc));
    return 1;
  }

  let c = camera.cam_new(k);
  if c.pos.x != 1.0 || c.pos.y != 2.0 || c.pos.z != 3.0 {
    io.println("m315: [CORRUPT] pos.x=" + convert.float_to_string(c.pos.x));
    return 2;
  }
  let l = camera.cam_len(c);
  if l < 3.7416 || l > 3.7417 {
    io.println("m315: [CORRUPT] len=" + convert.float_to_string(l));
    return 3;
  }

  // The enum's own module also uses geom structs (timeline.tl_vec).
  let n = timeline.tl_vec(2.5);
  if n.x != 2.5 || n.z != 2.5 {
    io.println("m315: [CORRUPT] tl_vec.x=" + convert.float_to_string(n.x));
    return 5;
  }

  io.println("m315_2139_enum_geom_collision: ok");
  return 0;
}
