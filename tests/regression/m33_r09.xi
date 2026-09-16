// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

enum WrappedResult { PresentOk(val: Int), PresentErr(msg: Str), NotPresent }
fn try_find(v: Int) -> WrappedResult {
  if v > 10 { return WrappedResult.PresentOk(v); }
  if v < 0 { return WrappedResult.PresentErr("negative"); }
  return WrappedResult.NotPresent;
}
fn main() -> Int {
  match try_find(20) { PresentOk(x) => { if x != 20 { return 1; } } PresentErr(_) => { return 2; } NotPresent => { return 3; } }
  match try_find(-5) { PresentOk(_) => { return 4; } PresentErr(_) => {} NotPresent => { return 5; } }
  match try_find(5) { PresentOk(_) => { return 6; } PresentErr(_) => { return 7; } NotPresent => {} }
  return 0;
}
