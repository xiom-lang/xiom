// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M65 regex-family fix regression: Vec[Option[T]] / Vec[Option[Struct]]
// element handling.
// - `Vec[Option[Int]].new()` used to size elements at 8 bytes (the bracketed
//   type arg fell through `type_arg_to_name` to "Int"), so push/read used the
//   scalar tag load and the Option payload was garbage.
// - `Vec[Option[Match]]`-style vectors additionally need the CONCRETE
//   Option__T size when the payload is a struct (32 bytes, not the erased 16).
// Covers: element-size resolution, the struct memcpy read path for container
// elements, and the Some ctor choosing the payload's concrete Option outside
// of a function whose return type disagrees (regex captures clang reject).
module m65_vec_option_elem

type VecPayload = {
  a: Int;
  b: Int;
}

fn wrap(v: Int) -> Option[Int] {
  return Some(v);
}

fn main() -> Int {
  var g = Vec[Option[Int]].new();
  g.push(Some(1));
  g.push(wrap(2));
  if g.len() != 2 {
    return 1;
  }
  match g[0] {
    Some(v) => { if v != 1 { return 2; } },
    None => { return 3; },
  }
  match g[1] {
    Some(v) => { if v != 2 { return 4; } },
    None => { return 5; },
  }

  // Struct payload: concrete Option__VecPayload (tag + 16-byte payload).
  var h = Vec[Option[VecPayload]].new();
  var p = VecPayload{ a: 7; b: 9; };
  h.push(Some(p));
  match h[0] {
    Some(q) => {
      if q.a != 7 { return 6; }
      if q.b != 9 { return 7; }
    },
    None => { return 8; },
  }
  return 0;
}
