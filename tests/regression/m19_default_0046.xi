// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0046

interface AllDefaults {
  fn one(&self) -> Int { return 1; }
  fn two(&self) -> Int { return 2; }
  fn three(&self) -> Int { return 3; }
}

type Empty = { }

fn Empty.one(self) -> Int { return 1; }

fn Empty.two(self) -> Int { return 2; }


fn Empty.three(&self) -> Int { return 33; }

fn main() -> Int {
  var e: Empty = Empty{ };
  if e.one() == 1 && e.two() == 2 && e.three() == 33 { return 0; }
  return 1;
}
