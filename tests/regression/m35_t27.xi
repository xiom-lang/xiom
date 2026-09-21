// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T27: Enum -- simple variants with match
enum Color { Red, Green, Blue }
fn is_red(c: Color) -> Bool { match c { Color.Red => true, _ => false } }
fn main() -> Int {
  if !is_red(Color.Red) { return 1; }
  if is_red(Color.Green) { return 2; }
  if is_red(Color.Blue) { return 3; }
  return 0;
}

