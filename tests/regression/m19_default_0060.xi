// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0060

interface Identifiable {
  fn id(&self) -> Int { return 0; }
}

type Custom = { val: Int; }

fn Custom.id(&self) -> Int { return val; }

fn main() -> Int {
  var c: Custom = Custom{ val: 99 };
  if c.id() == 99 { return 0; }
  return 1;
}
