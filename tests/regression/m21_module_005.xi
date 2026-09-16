// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_module_005
pub type Config = { port: Int; debug: Bool; }

  pub fn default_config() -> Config {
    return { port: 8080; debug: false; };
  }

  pub fn run() -> Int {
    var cfg = default_config();
    if cfg.port == 8080 { return 0; }
    return 1;
  }
use m21_module_005.run;
fn main() -> Int { return run(); }
