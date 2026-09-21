// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_complex_generic_003
type Box[T] = { value: T; }

  fn Box.get[T]() -> T { return self.value; }
  fn Box.set[T](v: T) { self.value = v; }

  pub fn run() -> Int {
    var b: Box[Int] = { value: 10; };
    b.set(99);
    if b.get() == 99 { return 0; }
    return 1;
  }
use m21_complex_generic_003.run;
fn main() -> Int { return run(); }
