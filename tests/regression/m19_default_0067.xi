// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0067

interface TryGet {
  fn try_get(&self) -> Option[Str] {
    var s = name();
    if s.len() > 0 { return Some(s); }
    return None;
  }
  fn name(&self) -> Str;
}

type Data = { val: Str; }

fn Data.name(&self) -> Str { return val; }

fn main() -> Int {
  var d: Data = Data{ val: "found" };
  var result = d.try_get();
  match result {
    Some(s) => if s != "found" { return 1; },
    None => return 2
  }
  return 0;
}
