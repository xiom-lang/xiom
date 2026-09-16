// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use repro_opt_contract;

fn main() -> Int {
  // BUG 28 #2 shape: ensures with Some-payload binding on an Option[Str] return
  var h = home_dir();
  if !h.is_some() { return 1; }
  var home = h.unwrap();
  if home.len() == 0 { return 2; }
  // BUG 28 #3 shape: second catalog hop + match + return
  var c = config_dir();
  if !c.is_some() { return 3; }
  var cfg = c.unwrap();
  if cfg != "C:\\Users\\test\\profile\\AppData" { return 4; }
  return 0;
}
