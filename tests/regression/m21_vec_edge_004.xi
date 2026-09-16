// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_vec_edge_004
pub fn run() -> Int {
    var v: Vec[Int] = [10, 20, 30];
    var val = v.pop();
    // stdlib Vec.pop returns Option[T]; verify the popped payload.
    var ok = false;
    match val {
      Some(x) => { if x == 30 { ok = true; } }
      None => {}
    }
    if ok && v.len() == 2 { return 0; }
    return 1;
  }
use m21_vec_edge_004.run;
fn main() -> Int { return run(); }
