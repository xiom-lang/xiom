// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0119

interface Counter {
  fn describe(&self) -> Int { return count(); }
  fn count(&self) -> Int;
}

type Type1 = { a: Int; }

fn Type1.describe(self) -> Int { return self.count(); }


fn Type1.count(&self) -> Int { return 1; }

type Type2 = { a: Int; b: Int; c: Int; }

fn Type2.count(&self) -> Int { return 3; }

fn main() -> Int {
  var t1: Type1 = Type1{ a: 0 };
  var t2: Type2 = Type2{ a: 0, b: 0, c: 0 };
  if t1.count() != 1 { return 1; }
  if t1.describe() != 1 { return 2; }
  if t2.count() != 3 { return 3; }
  if t2.describe() != 3 { return 4; }
  return 0;
}
