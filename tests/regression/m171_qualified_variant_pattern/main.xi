// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m171 lock (selfhost Phase 2 finding (f)): qualified enum-variant patterns
// ("Tree.Leaf") must satisfy match exhaustiveness. The parser stores the
// dotted name while the matcher compared against the bare variant, so both
// arms were flagged W000 'non-exhaustive'. Check-only: the recursive enum
// constructor temporaries hit finding (e) at runtime.
enum Tree {
  Leaf(v: Int),
  Node(l: Tree, r: Tree),
}

fn depth(t: Tree) -> Int {
  match t {
    Tree.Leaf(v) => { return 1; }
    Tree.Node(l, r) => { return 1 + depth(l) + depth(r); }
  }
}

fn main() -> Int {
  return 0;
}
