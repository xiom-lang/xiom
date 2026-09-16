// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0112

interface Classifier {
  fn classify(&self) -> Str { var v = value(); if v > 0 { if v > 100 { return "big"; } elif v > 50 { return "mid"; } else { return "small"; } } else { return "neg"; } }
  fn value(&self) -> Int;
}

type Data = { x: Int; }

fn Data.value(&self) -> Int { return x; }

fn main() -> Int {
  var d: Data = Data{ x: 75 };
  if d.value() != 75 { return 1; }
  if d.classify() != "mid" { return 2; }
  return 0;
}
