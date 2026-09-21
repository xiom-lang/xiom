// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-G03: Generic struct with type parameter
type Wrapper = { val: Int; }
fn make_wrapper(x: Int) -> Wrapper { return Wrapper{ val: x }; }
fn Wrapper.unwrap(self) -> Int { return val; }
fn main() -> Int {
  var w = make_wrapper(42);
  var v = w.unwrap();
  if v != 42 { return 1; }
  return 0;
}
