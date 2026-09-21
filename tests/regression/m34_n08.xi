// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N08: Struct with derive[Display] -- verify to_str() compiles and runs
type Label = { id: Int; text: Int; } derive[Display]
fn main() -> Int {
  var a = Label{ id: 42; text: 100; };
  var s = a.to_str();
  return 0;
}
