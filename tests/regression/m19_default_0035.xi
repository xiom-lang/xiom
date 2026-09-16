// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0035

interface Describable {
  fn describe(&self) -> Str {
    match name() {
      Some(n) => n,
      None => "unnamed"
    }
  }
  fn name(&self) -> Option[Str];
}

type Named = { label: Option[Str]; }

fn Named.name(&self) -> Option[Str] { return label; }

fn main() -> Int {
  var a: Named = Named{ label: Some("alpha") };
  var b: Named = Named{ label: None };
  if a.describe() != "alpha" { return 1; }
  if b.describe() != "unnamed" { return 2; }
  return 0;
}
