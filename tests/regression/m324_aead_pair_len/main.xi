// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m324 lock (REL-20261014-2123-stdlib): a function returning
// Result<(Vec[UInt8], Vec[UInt8]), Str> with `Ok((a, b))` must read back the
// tuple Vec lengths correctly at the CALLER (`pair.0.len()`), not
// 0xFFFFFFFF. Minimal trim of the stdlib crypto aes_encrypt_gcm shape.
module m324_aead_pair_len;

fn mkpair() -> Result<(Vec[UInt8], Vec[UInt8]), Str> {
  var a = Vec[UInt8].new();
  a.push(72u8); a.push(101u8); a.push(108u8); a.push(108u8); a.push(111u8);
  var b = Vec[UInt8].new();
  var i = 0;
  while i < 16 { b.push(i as UInt8); i = i + 1; }
  return Ok((a, b));
}

fn main() -> Int {
  let enc = mkpair();
  match enc {
    Ok(pair) => {
      if pair.0.len() != 5 { return 3; }
      if pair.1.len() != 16 { return 4; }
      if pair.0[0] != 72u8 { return 5; }
      if pair.1[15] != 15u8 { return 6; }
      return 0;
    },
    Err(_) => { return 7; },
  }
}
