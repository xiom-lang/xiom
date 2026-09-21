// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0013

interface Signable {
  fn sign(&self) -> Str {
    match value() {
      n if n > 0 => "positive",
      n if n < 0 => "negative",
      _ => "zero"
    }
  }
  fn value(&self) -> Int;
}

type IntVal = { x: Int; }

fn IntVal.value(&self) -> Int { return x; }

fn main() -> Int {
  var pos: IntVal = IntVal{ x: 3 };
  var neg: IntVal = IntVal{ x: -1 };
  var zero: IntVal = IntVal{ x: 0 };
  if pos.sign() == "positive" && neg.sign() == "negative" && zero.sign() == "zero" { return 0; }
  return 1;
}
