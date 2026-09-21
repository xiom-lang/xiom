// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0064

interface Calc {
  fn a(&self) -> Int { return 1; }
  fn b(&self) -> Int { return a() * 2; }
}

type Default = { val: Int; }

fn Default.b(self) -> Int { return self.a() * 2; }


fn Default.a(&self) -> Int { return val; }

type Override = { val: Int; }

fn Override.a(&self) -> Int { return val; }

fn main() -> Int {
  var d: Default = Default{ val: 5 };
  var o: Override = Override{ val: 10 };
  if d.a() == 5 && d.b() == 10 && o.a() == 10 && o.b() == 20 { return 0; }
  return 1;
}
