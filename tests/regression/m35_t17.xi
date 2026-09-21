// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T17: Interface on each type -- multiple interfaces, cross-type impl
interface Identifiable { fn id(self) -> Int; }
interface Describable { fn desc(self) -> Str; }
interface Measurable { fn size(self) -> Int; }
type Node = { id: Int; label: Str; }
impl Identifiable for Node { fn id(self) -> Int { return self.id; } }
impl Describable for Node { fn desc(self) -> Str { return self.label; } }
impl Measurable for Node { fn size(self) -> Int { return self.label.len() as Int; } }
type Counter = { count: Int; }
impl Identifiable for Counter { fn id(self) -> Int { return self.count; } }
impl Measurable for Counter { fn size(self) -> Int { return self.count; } }
type Tagged = { tag: Str; value: Int; }
impl Identifiable for Tagged { fn id(self) -> Int { return self.value; } }
impl Describable for Tagged { fn desc(self) -> Str { return self.tag; } }
fn main() -> Int {
  var n = Node{ id: 1; label: "node"; };
  if n.id() != 1 { return 1; }
  if n.desc() != "node" { return 2; }
  if n.size() != 4 { return 3; }
  var c = Counter{ count: 99; };
  if c.id() != 99 { return 4; }
  if c.size() != 99 { return 5; }
  var t = Tagged{ tag: "tag"; value: 42; };
  if t.id() != 42 { return 6; }
  if t.desc() != "tag" { return 7; }
  return 0;
}

