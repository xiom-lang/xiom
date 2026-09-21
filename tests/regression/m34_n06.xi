// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N06: Enum with derive[Clone] -- enum variant clone
enum State { On, Off, Unknown } derive[Clone]
fn main() -> Int {
  var a = State.On;
  var b = a.clone();
  var c = State.Unknown;
  var d = c.clone();
  if b == State.On && d == State.Unknown { return 0; }
  return 1;
}
