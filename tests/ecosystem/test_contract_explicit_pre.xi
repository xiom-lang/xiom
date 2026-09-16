// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// Contract with explicit @pre
module tests.ecosystem.test_contract_explicit_pre

pub type Widget = { count: Int; active: Bool; }

requires w.active
ensures result == w@pre.count + 1
fn Widget.tick(w: &mut Widget) -> Int {
  w.count = w.count + 1;
  return w.count;
}

fn main() -> Int {
  var w = Widget{ count: 0, active: true };
  let r = Widget.tick(&mut w);
  if r == 1 { return 0; }
  return 1;
}
