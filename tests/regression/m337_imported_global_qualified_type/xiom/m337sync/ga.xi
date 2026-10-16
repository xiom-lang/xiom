// m337 (XVC-C-17): same-leaf collision owner `ga` -- Widget{alpha} (one
// field) collides with gb.Widget{beta;gamma} (two fields). The qualified
// spelling of the global's type must bind THIS module's registry key, not
// the first same-leaf candidate (XVC-C-14 class).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module xiom.m337sync.ga;

pub type Widget = { alpha: Int; };

pub fn make_ga() -> Int {
  return 7;
}
