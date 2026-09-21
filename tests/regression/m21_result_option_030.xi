// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_030
type Config = { port: Option[Int]; host: Option[Int]; }

  pub fn run() -> Int {
    var cfg: Config = { port: Some(8080); host: None; };
    match cfg.port {
      Some(p) => if p == 8080 { return 0; },
      None => return 1,
    }
  }
use m21_result_option_030.run;
fn main() -> Int { return run(); }
