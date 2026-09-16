// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N2-16: 10-deep while-with-if nesting -- alternating while/if depth stress
fn main() -> Int {
  var i = 0;
  var ok = 1;
  while i < 1 {
    if ok == 1 {
      while i < 1 {
        if ok == 1 {
          while i < 1 {
            if ok == 1 {
              while i < 1 {
                if ok == 1 {
                  while i < 1 {
                    if ok == 1 {
                      ok = ok + 0;
                    } else { ok = 0; }
                    i = i + 1;
                  }
                } else { ok = 0; }
              }
            } else { ok = 0; }
          }
        } else { ok = 0; }
      }
    } else { ok = 0; }
  }
  if ok == 1 { return 0; }
  return 1;
}
