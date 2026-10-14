// m316 (REL-20261014-1158-engine) consumer: the imported module veh must
// expose ship_ready / crate_ready / ship_shorthand regardless of the
// spawn-named field. Pre-fix the module AST lost the fns at parse time and
// the consumers saw "cannot call 'ship_ready' on this expression" (silent
// export drop).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m316_spawn_shadow

use veh;

fn main() -> Int {
  if veh.ship_ready(true) != 7 { return 1; }
  if veh.ship_ready(false) != 0 { return 2; }
  if veh.crate_ready(true) != 5 { return 3; }
  if veh.crate_ready(false) != 0 { return 4; }
  if veh.ship_shorthand(true) != 3 { return 5; }
  if veh.ship_shorthand(false) != 0 { return 6; }
  return 0;
}
