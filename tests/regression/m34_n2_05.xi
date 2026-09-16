// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N2-05: Nested enum in enum in enum -- 3 levels of enum payload nesting
type InnerE = enum { I(val: Int), }
type MiddleE = enum { M(inner: InnerE), }
type OuterE = enum { O(mid: MiddleE), }

fn main() -> Int {
  var e = OuterE.O(MiddleE.M(InnerE.I(77)));
  match e {
    OuterE.O(m) => {
      match m {
        MiddleE.M(i) => {
          match i {
            InnerE.I(v) => { if v == 77 { return 0; } }
          }
        }
      }
    }
  }
  return 1;
}
