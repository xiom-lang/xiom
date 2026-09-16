// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-O09: ? with if guard -- ? guarded by condition
fn maybe_val(flag: Bool) -> Result[Int, Str] {
  if flag { return Ok(100); }
  return Err("off");
}
fn guarded(flag: Bool, threshold: Int) -> Result[Int, Str] {
  if flag {
    var v = maybe_val(true)?;
    if v > threshold { return Ok(v); }
    return Ok(0);
  }
  return Err("disabled");
}
fn main() -> Int {
  match guarded(true, 50) { Ok(v) => { if v != 100 { return 1; } } Err(_) => { return 2; } }
  match guarded(true, 150) { Ok(v) => { if v != 0 { return 3; } } Err(_) => { return 4; } }
  match guarded(false, 50) { Ok(_) => { return 5; } Err(_) => {} }
  return 0;
}
