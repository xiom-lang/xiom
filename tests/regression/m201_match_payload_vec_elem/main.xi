// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m201 (stdlib p_multipart_parse_name): a match-bound Result[Vec[Struct], _]
// payload must keep the concrete Vec element type. Pre-fix the `Ok(out)`
// binding recorded no element type, so `out[0]` took the runtime elem-size
// switch (scalar i64 default) instead of the struct memcpy path and every
// Part field read garbage (-1 / 0; multipart_parse probe rc 1).
type Part = {
  name: Str;
  value: Str;
}

fn mk() -> Result[Vec[Part], Str] {
  var v = Vec[Part].new();
  v.push(Part{ name: "f", value: "v" });
  return Ok(v);
}

fn main() -> Int {
  match mk() {
    Ok(out) => {
      if out.len() != 1 { return 2; }
      if !(out[0].name == "f") { return 1; }
      if !(out[0].value == "v") { return 4; }
    },
    Err(_) => { return 3; },
  }
  return 0;
}
