// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// m75 canon: canonical same-leaf module the shim delegates to.
module m75canon.base32

pub fn encode(x: Int) -> Int {
  return x * 2;
}

pub fn name() -> Str {
  return "canon";
}

pub fn decode(x: Int) -> Result[Int, Str] {
  if x < 0 {
    return Err("neg");
  }
  return Ok(x * 2);
}
