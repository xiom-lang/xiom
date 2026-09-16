// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N2-20: Deep match nesting with multi-variant enums -- 3-level pattern depth
type X1 = enum { A(val: Int), B(val: Int), }
type X2 = enum { W(a: X1), X(a: X1), }
type X3 = enum { P(x2: X2), Q(x2: X2), }

fn main() -> Int {
  var e1 = X3.P(X2.W(X1.A(7)));
  match e1 {
    X3.P(x2) => {
      match x2 {
        X2.W(x1) => {
          match x1 {
            X1.A(v) => { if v != 7 { return 1; } }
            X1.B(v) => { if v != 7 { return 1; } }
          }
        }
        X2.X(x1) => {
          match x1 {
            X1.A(v) => { if v != 7 { return 1; } }
            X1.B(v) => { if v != 7 { return 1; } }
          }
        }
      }
    }
    X3.Q(x2) => {
      match x2 {
        X2.W(x1) => {
          match x1 {
            X1.A(v) => { if v != 7 { return 1; } }
            X1.B(v) => { if v != 7 { return 1; } }
          }
        }
        X2.X(x1) => {
          match x1 {
            X1.A(v) => { if v != 7 { return 1; } }
            X1.B(v) => { if v != 7 { return 1; } }
          }
        }
      }
    }
  }
  return 0;
}
