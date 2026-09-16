// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m19_default_0075

interface Identifiable {
  fn id(&self) -> Str { return name(); }
  fn name(&self) -> Str;
}

interface Greeter {
  fn greet(&self) -> Str { return "Hello " + name(); }
  fn name(&self) -> Str;
}

interface Describable {
  fn desc(&self) -> Str { return "[" + name() + "]"; }
  fn name(&self) -> Str;
}

type Person = { n: Str; }

fn Person.id(self) -> Str { return self.name(); }

fn Person.desc(self) -> Str { return "[" + self.name() + "]"; }


fn Person.name(&self) -> Str { return n; }

fn Person.greet(&self) -> Str { return "Hi " + name(); }

type Agent = { n: Str; }

fn Agent.name(&self) -> Str { return n; }

fn main() -> Int {
  var p: Person = Person{ n: "Alice" };
  var a: Agent = Agent{ n: "Bot" };
  if p.name() != "Alice" { return 1; }
  if p.id() != "Alice" { return 2; }
  if p.greet() != "Hi Alice" { return 3; }
  if p.desc() != "[Alice]" { return 4; }
  if a.name() != "Bot" { return 5; }
  if a.id() != "Bot" { return 6; }
  if a.greet() != "Hello Bot" { return 7; }
  if a.desc() != "[Bot]" { return 8; }
  return 0;
}
