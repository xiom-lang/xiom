// M34-N07: Nested struct derive[Clone] -- clone propagates through nesting
type Leaf = { val: Int; } derive[Clone]
type Branch = { left: Leaf; right: Leaf; } derive[Clone]
fn main() -> Int {
  var a = Branch{ left: Leaf{ val: 1 }; right: Leaf{ val: 2 }; };
  var b = a.clone();
  if b.left.val == 1 && b.right.val == 2 && a.left.val == 1 { return 0; }
  return 1;
}
