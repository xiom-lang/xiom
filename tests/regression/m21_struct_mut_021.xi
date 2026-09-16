// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_021
type Flag = { active: Bool; ready: Bool; }
fn main() -> Int {
  var f: Flag = Flag{ active: false; ready: false; };
  f.active = true;
  f.ready = true;
  if f.active && f.ready { return 0; }
  return 1;
  return 1;
}
