// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Module B: aggregate `g` with the same bare name as module A's scalar.
// Pre-fix the catalog injector qualified only immutable consts, so this
// definition was deduped away and `get_b()` read A's `g` as a Box (0.00).

module xiom.coll.b

pub type Box = {
  x: Float64;
}

pub var g: Box = Box{ x: 0.5 };

pub fn get_b() -> Float64 { return g.x; }
