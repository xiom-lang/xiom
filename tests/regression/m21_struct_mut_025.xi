// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_025
type Operation = { status: Result[Int, Int]; id: Int; }
fn main() -> Int {
  var op: Operation = Operation{ status: Err(-1); id: 0; };
  op.status = Ok(200);
  op.id = 7;
  match op.status {
    Ok(code) => if code == 200 && op.id == 7 { return 0; },
    Err(_) => return 1,
  }
  return 1;
}
