// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0025

interface Titled {
  fn formal(&self) -> Str { return "Mr. " + name(); }
  fn name(&self) -> Str;
}

type Doctor = { surname: Str; }

fn Doctor.name(&self) -> Str { return surname; }

fn Doctor.formal(&self) -> Str { return "Dr. " + name(); }

fn main() -> Int {
  var d: Doctor = Doctor{ surname: "Smith" };
  if d.name() == "Smith" && d.formal() == "Dr. Smith" { return 0; }
  return 1;
}
