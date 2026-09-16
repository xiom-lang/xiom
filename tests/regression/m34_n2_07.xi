// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N2-07: 4-level nested match -- E1 inside E2 inside E3 inside E4
type E1 = enum { A1(val: Int), }
type E2 = enum { A2(val: E1), }
type E3 = enum { A3(val: E2), }
type E4 = enum { A4(val: E3), }

fn main() -> Int {
  var e = E4.A4(E3.A3(E2.A2(E1.A1(55))));
  match e {
    E4.A4(e3) => {
      match e3 {
        E3.A3(e2) => {
          match e2 {
            E2.A2(e1) => {
              match e1 {
                E1.A1(v) => { if v == 55 { return 0; } }
              }
            }
          }
        }
      }
    }
  }
  return 1;
}
