// XIOM -- phase1_error
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

fn safe_divide(a: Float64, b: Float64) -> Result[Float64, Str] {
  if b == 0.0 {
    return Err("division by zero");
  }
  return Ok(a / b);
}

fn main() -> Int {
  let r = safe_divide(10.0, 2.0);
  match r {
    Ok(v) => { return 0; }
    Err(e) => { return -1; }
  }
}
