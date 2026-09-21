// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module repro_opt_contract

pub fn var_opt(name: Str) -> Option[Str] {
  if name.len() > 0 {
    return Some("C:\\Users\\test\\profile");
  }
  return None;
}

pub fn home_dir() -> Option[Str]
ensures: result is Some => result.len() > 0 {
  return var_opt("USERPROFILE");
}

pub fn config_dir() -> Option[Str] {
  let h = home_dir();
  match h {
    Some(home) => Some(home + "\\AppData"),
    None => None,
  }
}
