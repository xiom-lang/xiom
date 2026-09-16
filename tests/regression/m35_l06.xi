// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L06: Struct with Str fields -- verify string layout in struct
type Labeled = { key: Str; value: Int; note: Str; }

fn main() -> Int {
  var lab = Labeled{ key: "count"; value: 100; note: "units"; };
  if lab.value != 100 { return 1; }
  if lab.key == "count" { return 0; }
  return 2;
}
