// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0030

interface PiProvider {
  fn pi(&self) -> Float64 { return 3.14159; }
  fn describe(&self) -> Str;
}

type Circle = { label: Str; }

fn Circle.pi(self) -> Float64 { return 3.14159; }


fn Circle.describe(&self) -> Str { return label; }

fn main() -> Int {
  var c: Circle = Circle{ label: "unit" };
  if c.describe() == "unit" && c.pi() == 3.14159 { return 0; }
  return 1;
}
