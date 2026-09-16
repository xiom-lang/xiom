// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_008
enum ErrorKind { NotFound, Permission, Timeout }

  type AppError = { kind: ErrorKind; msg: Str; code: Int; }

  pub fn run() -> Int {
    var r: Result[Int, AppError] = Err({ kind: ErrorKind.NotFound; msg: "missing"; code: 404; });
    match r {
      Ok(_) => return 1,
      Err(e) => if e.code == 404 { return 0; },
    }
  }
use m21_result_option_008.run;
fn main() -> Int { return run(); }
