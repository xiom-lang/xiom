// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N2-15: 8-level type alias chain -- T0 -> T1 -> T2 -> T3 -> T4 -> T5 -> T6 -> T7
type T0 = Int;
type T1 = T0;
type T2 = T1;
type T3 = T2;
type T4 = T3;
type T5 = T4;
type T6 = T5;
type T7 = T6;

fn ident(x: T7) -> T7 { return x; }

type Container = { val: T7; }

fn main() -> Int {
  var a: T7 = 42;
  var b = ident(a);
  var c = Container{ val: b; };
  if c.val == 42 { return 0; }
  return 1;
}
