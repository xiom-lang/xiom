// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0014

interface Plain {
  fn alpha(&self) -> Int;
  fn beta(&self) -> Str;
}

type Data = { a: Int; b: Str; }

fn Data.alpha(&self) -> Int { return a; }

fn Data.beta(&self) -> Str { return b; }

fn main() -> Int {
  var d: Data = Data{ a: 1, b: "two" };
  if d.alpha() == 1 && d.beta() == "two" { return 0; }
  return 1;
}
