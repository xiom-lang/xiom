// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0118

interface FullName {
  fn full_name(&self) -> Str { return first() + " " + last(); }
  fn first(&self) -> Str;
  fn last(&self) -> Str;
}

type Person = { f: Str; l: Str; }

fn Person.full_name(self) -> Str { return self.first() + " " + self.last(); }


fn Person.first(&self) -> Str { return f; }
fn Person.last(&self) -> Str { return l; }

fn main() -> Int {
  var p: Person = Person{ f: "John", l: "Doe" };
  if p.first() != "John" { return 1; }
  if p.last() != "Doe" { return 2; }
  if p.full_name() != "John Doe" { return 3; }
  return 0;
}
