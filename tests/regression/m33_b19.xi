// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-B19: Return owned from function -- function produces new value, caller owns it
type Data = { val: Int; }
fn make_data(v: Int) -> Data { return Data{ val: v; }; }
fn extract(d: Data) -> Int { return d.val; }
fn main() -> Int {
  var d = make_data(99);
  var v = extract(d);
  if v == 99 { return 0; }
  return 1;
}
