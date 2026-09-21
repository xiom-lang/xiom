// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn unwrap_or[T](opt: Option[T], default: T) -> T {
  if opt.is_some { return opt.value; }
  return default;
}

fn main() -> Int {
  var x = Some(42);
  return unwrap_or(x, 0);
}
