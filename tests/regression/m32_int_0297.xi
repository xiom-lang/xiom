// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Struct with Int32 field mutated through Int
type Block = { addr: Int32; len: Int16; }
fn main() -> Int {
  var b: Block = Block{ addr: 300000000; len: 1000 as Int16; };
  var a: Int = b.addr as Int;
  if a == 300000000 { return 0; }
  return 1;
}
