// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m298 first module of the same-leaf triad (the "package" role: it sorts
// first, so pre-fix its BTree claimed the bare key while the second module's
// BTree declaration was silently skipped). Its BTreeNode deliberately has NO
// payloads field -- that is what made the loser's payloads.push/pop a C001.
module xiom.aa.btree;

pub type NodeStore = {
  nodes: Vec[BTreeNode];
}

pub type BTree = {
  root: Int;
  order: Int;
  store: NodeStore;
}

pub type BTreeNode = {
  keys: Vec[Int];
  values: Vec[Int];
}

pub fn atree_new(order: Int) -> BTree {
  return BTree{ root: 0; order: order;
                store: NodeStore{ nodes: Vec[BTreeNode].new() } };
}

pub fn atree_bump(tree: &mut BTree) -> Int {
  let n = tree.store.nodes.len();
  tree.root = tree.root + 1;
  return tree.root + n;
}
