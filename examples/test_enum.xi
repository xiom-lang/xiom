// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module enums_mod {
  pub enum Direction { North, South, East, West }

  pub fn go_north() -> Int {
    if Direction.North != Direction.South { return 1; }
    return 0;
  }

  pub fn run_test() -> Int { return go_north(); }
}

use enums_mod.run_test;

fn main() -> Int { return run_test(); }
