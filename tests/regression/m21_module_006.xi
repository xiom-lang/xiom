// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_module_006
pub enum Status { Ok, Warn, Error }

  pub fn check(code: Int) -> Status {
    if code == 0 { return Status.Ok; }
    elif code < 0 { return Status.Error; }
    return Status.Warn;
  }

  pub fn run() -> Int {
    var s = check(0);
    match s {
      Status.Ok => return 0,
      _ => return 1,
    }
  }
use m21_module_006.run;
fn main() -> Int { return run(); }
