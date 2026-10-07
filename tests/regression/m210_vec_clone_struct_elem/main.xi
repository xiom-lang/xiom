// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m210 (packages struct-clone relay): `let w = v.clone()` over Vec[Struct]
// must keep the element type so `w[i]` takes the struct read path. Pre-fix
// the binding recorded no element type, `w[1]` fell to the runtime elem-size
// scalar switch (inttoptr of the element's first field) and aborted
// 0xC0000005 (probe_struct_clone).
type Pair = {
  a: Int;
  b: Str;
}

fn main() -> Int {
  var v: Vec[Pair] = Vec[Pair].new();
  v.push(Pair{ a: 1, b: "one" });
  v.push(Pair{ a: 2, b: "two" });
  let w = v.clone();
  if w.len() != 2 { return 1; }
  if w[1].a != 2 { return 2; }
  if !(w[1].b == "two") { return 3; }
  // Deep copy: growing the clone must not disturb the original buffer.
  w.push(Pair{ a: 3, b: "three" });
  if w.len() != 3 { return 4; }
  if v.len() != 2 { return 5; }
  if v[0].a != 1 { return 6; }
  return 0;
}
