// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m310_port_args_helper.helper

extern "C" {
  fn m310_helper_bump(x: Int) -> Int;
}

pub fn bump(x: Int) -> Int {
  let v = unsafe { m310_helper_bump(x) };
  return v;
}
