// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-Y03: invariant struct + generic + match + unsafe + module
type Node = { val: Int; next: Int; invariant: val >= 0; }
enum Traversal { Sum, Product, Count }
fn traverse[T](n: Node, kind: Traversal) -> Int
  ensures: result >= 0
{
  match kind {
    Sum => 43,
    Product => { var acc = 1; unsafe { acc = acc * 7; } return acc; },
    Count => { var x = 0; x = x + 3; return x; },
  }
}
module graph {
  pub fn run(n: Node, k: Traversal) -> Int { return traverse(n, k); }
  pub fn val(n: Node) -> Int { return n.val; }
}
use graph.run;
use graph.val;
fn main() -> Int {
  var n = Node{ val: 42; next: 0; };
  var r1 = run(n, Traversal.Sum);
  var r2 = run(n, Traversal.Product);
  var r3 = run(n, Traversal.Count);
  if r1 == 43 && r2 == 7 && r3 == 3 { return 0; }
  return 1;
}
