// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// CTFE Phase A -- Builtins with Struct Types
// Tests sizeof, align_of, field_offset on user-defined structs

type Point2D = {
  x: Float64;
  y: Float64;
}

type Mixed = {
  a: Int8;
  b: Int32;
  c: Float64;
}

const SIZEOF_POINT: Int = sizeof::<Point2D>();
const ALIGNOF_POINT: Int = align_of::<Point2D>();
const OFFSET_X: Int = field_offset::<Point2D>("x");
const OFFSET_Y: Int = field_offset::<Point2D>("y");

fn main() -> Int {
  // Point2D = { Float64, Float64 } -> 16 bytes
  if SIZEOF_POINT != 16 { return 1; }
  // Float64 alignment = 8
  if ALIGNOF_POINT != 8 { return 2; }
  // x is first field -> offset 0
  if OFFSET_X != 0 { return 3; }
  // y is after Float64 -> offset 8
  if OFFSET_Y != 8 { return 4; }
  return 0;
}
