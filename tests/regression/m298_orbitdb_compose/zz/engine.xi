// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m298 second module of the same-leaf triad (the "engine" role). Its BTree
// shape string is identical to xiom.aa.btree's, which is exactly what put it
// under the facade exemption; its NodeStore/BTreeNode shapes differ (free /
// payloads fields), so those qualified. Pre-fix, zz.engine's bodies resolved
// `tree.store` through xiom.aa.btree's BTree meta and `child` came out as
// xiom.aa.btree.BTreeNode -- `child.payloads.push/pop` was then a call to a
// non-existent method -> C001.
module xiom.zz.engine;

pub type NodeStore = {
  nodes: Vec[BTreeNode];
  free: Int;
}

pub type BTree = {
  root: Int;
  order: Int;
  store: NodeStore;
}

pub type BTreeNode = {
  keys: Vec[Int];
  values: Vec[Int];
  payloads: Vec[Vec[Int]];
}

fn new_node() -> BTreeNode {
  return BTreeNode{ keys: Vec[Int].new(); values: Vec[Int].new();
                    payloads: Vec[Vec[Int]].new() };
}

pub fn engine_new(order: Int) -> BTree {
  return BTree{ root: 0; order: order;
                store: NodeStore{ nodes: Vec[BTreeNode].new(); free: 0 } };
}

// The exact pre-fix failing shape: a node read out of `tree.store.nodes`
// (whose declared element type is the bare leaf BTreeNode) followed by a
// payload-vector push/pop on it.
fn engine_touch(tree: &mut BTree) -> Int {
  var child = tree.store.nodes[0];
  child.payloads.push(Vec[Int].new());
  let _ = child.payloads.pop();
  return child.keys.len();
}

pub fn engine_insert(tree: &mut BTree, v: Int) -> Int {
  var node = new_node();
  node.keys.push(v);
  node.values.push(v);
  node.payloads.push(Vec[Int].new());
  tree.store.nodes.push(node);
  tree.root = tree.root + 1;
  return engine_touch(tree);
}

pub fn engine_trim(tree: &mut BTree) -> Int {
  var child = tree.store.nodes[0];
  let _ = child.payloads.pop();
  return child.payloads.len();
}
