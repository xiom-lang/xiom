// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module repro_gcm

pub fn aes_encrypt_gcm(a: Vec[UInt8], b: Vec[UInt8]) -> Result[(Vec[UInt8], Vec[UInt8]), Str] {
  return Ok((a, b));
}

pub fn gcm_passthrough(a: Vec[UInt8], b: Vec[UInt8]) -> (Vec[UInt8], Vec[UInt8]) {
  return (a, b);
}
