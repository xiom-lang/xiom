// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-E11: Match with block expressions (multi-statement arms)
enum Action { Add(x: Int), Sub(x: Int), Mul(x: Int), Noop }
fn apply(act: Action, base: Int) -> Int {
  match act {
    Add(x) => {
      var result = base + x;
      return result;
    }
    Sub(x) => {
      var result = base - x;
      return result;
    }
    Mul(x) => {
      var result = base * x;
      return result;
    }
    Noop => { return base; }
  }
}
fn main() -> Int {
  if apply(Action.Add(5), 10) != 15 { return 1; }
  if apply(Action.Sub(3), 10) != 7 { return 2; }
  if apply(Action.Mul(2), 10) != 20 { return 3; }
  if apply(Action.Noop, 10) != 10 { return 4; }
  return 0;
}
