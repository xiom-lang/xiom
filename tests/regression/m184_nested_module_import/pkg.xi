// m184 peer module (packages relay): must stay visible when a sibling file
// declares the nested module pkg.tests.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module pkg

pub fn val() -> Int {
  return 7;
}
