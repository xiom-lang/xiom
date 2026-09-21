// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L28: Packed struct pattern -- tightly packed fields with verification
type Packed = { flag: Bool; tag: Char; count: Int; }

fn main() -> Int {
  var p = Packed{ flag: true; tag: 'K'; count: 255; };
  if p.flag != true { return 1; }
  if p.tag != 'K' { return 2; }
  if p.count != 255 { return 3; }
  if p.flag {
    var ch: Char = p.tag;
    if ch == 'K' { return 0; }
  }
  return 4;
}
