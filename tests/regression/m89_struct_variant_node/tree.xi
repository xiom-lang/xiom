// Enum whose `Node` VARIANT payload is (value,left,right). Pre-fix the
// literal in `build` compiled against m89.types' struct Node (value,
// children) and stored a %struct.Vec into the tree payload (bench BST).
module m89.tree

pub enum Tree {
  Leaf
  Node(value: Int, left: Tree, right: Tree)
}

pub fn build() -> Tree {
  return Node(value: 1, left: Leaf, right: Leaf);
}

pub fn root_value(t: &Tree) -> Int {
  match t {
    Leaf => return -1,
    Node(value: v, left: _, right: _) => return v
  }
  return -1;
}
