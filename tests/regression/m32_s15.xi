// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-S15: Struct composition -- build up from sub-structs with field manipulation
type Header = { id: Int; version: Int; }
type Body = { message: Str; count: Int; }
type Packet = { header: Header; body: Body; }
fn main() -> Int {
  var h = Header{ id: 1; version: 2; };
  var b = Body{ message: "ping"; count: 3; };
  var p = Packet{ header: h; body: b; };
  p.header.id = 100;
  p.body.count = p.body.count + 10;
  if p.header.id == 100 && p.body.count == 13 && p.header.version == 2 { return 0; }
  return 1;
}
