// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0125

interface Identifiable {
  fn id(&self) -> Str { return name() + "#" + key().to_str(); }
  fn name(&self) -> Str;
  fn key(&self) -> Int { return 0; }
}

interface Greeter {
  fn greet(&self) -> Str { return "Hi " + name(); }
  fn name(&self) -> Str;
}

interface Describable {
  fn desc(&self) -> Str { return "[" + id() + "]: " + greet(); }
}

type Person = { tag: Str; }

fn Person.id(self) -> Str { return self.name() + "#" + self.key().to_str(); }

fn Person.greet(self) -> Str { return "Hi " + self.name(); }

fn Person.desc(self) -> Str { return "[" + self.id() + "]: " + self.greet(); }


fn Person.name(&self) -> Str { return tag; }
fn Person.key(&self) -> Int { return 42; }

fn main() -> Int {
  var p: Person = Person{ tag: "name" };
  if p.name() != "name" { return 1; }
  if p.key() != 42 { return 2; }
  if p.id() != "name#42" { return 3; }
  if p.greet() != "Hi name" { return 4; }
  if p.desc() != "[name#42]: Hi name" { return 5; }
  return 0;
}
