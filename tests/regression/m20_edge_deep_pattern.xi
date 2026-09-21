// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

enum Tree { Leaf(val: Int), Node(left: Int, right: Int) }
fn sum(t: Tree) -> Int { match t { Leaf(v) => { return v; } Node(l, r) => { return l + r; } } }
fn main() -> Int { if sum(Tree.Leaf(42)) != 42 { return 1; } if sum(Tree.Node(10, 20)) != 30 { return 2; } return 0; }