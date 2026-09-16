// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// BUG 55 facet-2 regression: Some-payload reads from generic-method
// call scrutinees (`match v.get(i) { Some(cur) => ... }`) bound literal 0
// (no scrutinee alloca -- the generic fn's return type didn't resolve),
// and Vec.set (generic, never inlined) called the never-emitted def ->
// zero-param stub -> swap-style sorts silently no-oped.
module m37_bug55_payload_loop
use xiom.collections;

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(5);
  v.push(3);
  var i: Int = 1;
  while i < v.len() {
    var j = i;
    while j > 0 {
      match v.get(j) {
        Some(cur) => {
          match v.get(j - 1) {
            Some(prev) => {
              if cur < prev {
                v.set(j, prev);
                v.set(j - 1, cur);
              }
            }
            None => {}
          }
        }
        None => {}
      }
      j = j - 1;
    }
    i = i + 1;
  }
  if v[0] != 3 { return 1; }
  if v[1] != 5 { return 2; }
  return 0;
}
