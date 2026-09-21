// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn unwrap_or(o: Option[Int], default: Int) -> Int {
  match o { Some(v) => v, None => default }
}
fn main() -> Int {
  if unwrap_or(Some(42), 0) != 42 { return 1; }
  if unwrap_or(None, 99) != 99 { return 2; }
  return 0;
}
