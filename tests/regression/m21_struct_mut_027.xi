// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_027
type Item = { id: Int; weight: Float64; }

pub fn run() -> Int {
    var items: Vec[Item] = [{ id: 1; weight: 2.5; }, { id: 2; weight: 3.0; }];
    items[0].weight = 4.5;
    items[1].id = 99;
    if items[0].weight == 4.5 && items[1].id == 99 { return 0; }
    return 1;
  }
use m21_struct_mut_027.run;
fn main() -> Int { return run(); }
