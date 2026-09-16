// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_039
enum AppError { Io, Parse, Auth, Timeout, Internal }

  pub fn run() -> Int {
    var r: Result[Int, AppError] = Err(AppError.Timeout);
    match r {
      Ok(_) => return 1,
      Err(e) => match e {
        AppError.Timeout => return 0,
        _ => return 1,
      },
    }
  }
use m21_result_option_039.run;
fn main() -> Int { return run(); }
