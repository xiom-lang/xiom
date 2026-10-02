// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m174 lock (stdlib relay, p_struct_literal_field_order): struct literals
// whose fields are written out of declaration order must store each value
// into its DECLARED slot by NAME. Codegen walked the supplied list and wrote
// slot i, so `P{ z; y; x }` against `{x;y;z}` produced p.x == 3.0 (this
// silently scrambled every Euler-derived quaternion in the stdlib).
type P = {
  x: Float64;
  y: Float64;
  z: Float64;
}

fn mk_reordered() -> P {
  return P{ z: 3.0; y: 2.0; x: 1.0; };
}

// Mixed scalar/Str/Vec fields, supplied in a different order again.
type Mix = {
  id: Int;
  name: Str;
  vals: Vec[Int];
}

fn main() -> Int {
  var p = mk_reordered();
  if p.x != 1.0 { return 11; }
  if p.y != 2.0 { return 12; }
  if p.z != 3.0 { return 13; }

  var m = Mix{ vals: Vec[Int].new(); name: "n"; id: 7; };
  if m.id != 7 { return 14; }
  if m.name != "n" { return 15; }
  if m.vals.len() != 0 { return 16; }

  // Guard: declaration-order literals keep working.
  var p2 = P{ x: 1.0; y: 2.0; z: 3.0; };
  if p2.x != 1.0 { return 17; }
  if p2.y != 2.0 { return 18; }
  if p2.z != 3.0 { return 19; }
  return 0;
}
