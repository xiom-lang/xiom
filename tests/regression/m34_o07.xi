// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-O07: ? in function returning Option -- match-based Option propagation
fn try_get(flag: Bool) -> Option[Int] {
  if flag { return Some(42); }
  return None;
}
fn unwrap_or_default(o: Option[Int]) -> Int {
  match o { Some(v) => v, None => -1 }
}
fn chain_option(flag1: Bool, flag2: Bool) -> Option[Int] {
  match try_get(flag1) {
    Some(v1) => {
      match try_get(flag2) {
        Some(v2) => Some(v1 + v2),
        None => None
      }
    }
    None => None
  }
}
fn main() -> Int {
  match chain_option(true, true) { Some(v) => { if v != 84 { return 1; } } None => { return 2; } }
  match chain_option(true, false) { Some(_) => { return 3; } None => {} }
  match chain_option(false, true) { Some(_) => { return 4; } None => {} }
  if unwrap_or_default(Some(100)) != 100 { return 5; }
  if unwrap_or_default(None) != -1 { return 6; }
  return 0;
}
