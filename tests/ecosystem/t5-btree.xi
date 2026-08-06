use xiom.io;
use xiom.math;
use xiom.time;

type BTreeNode = {
  num_keys: Int;
  is_leaf: Int;
  parent_offset: Int;
  k0: Int;
  k1: Int;
  k2: Int;
  v0: Int;
  v1: Int;
  v2: Int;
  c0: Int;
  c1: Int;
  c2: Int;
  c3: Int;
}

const NODE_POOL_SIZE: Int = 10000;

type BTree = {
  root_offset: Int;
  num_nodes: Int;
  nodes: Vec[BTreeNode];
}

fn BTree.new() -> BTree {
  return BTree{
    root_offset: 0,
    num_nodes: 0,
    nodes: Vec[BTreeNode].with_capacity(NODE_POOL_SIZE)
  };
}

fn BTreeNode.new_leaf() -> BTreeNode {
  return BTreeNode{
    num_keys: 0,
    is_leaf: 1,
    parent_offset: -1,
    k0: 0, k1: 0, k2: 0,
    v0: 0, v1: 0, v2: 0,
    c0: -1, c1: -1, c2: -1, c3: -1
  };
}

fn BTree.get_key(node_idx: Int, i: Int) -> Int {
  if i == 0 { return nodes[node_idx].k0; }
  if i == 1 { return nodes[node_idx].k1; }
  return nodes[node_idx].k2;
}

fn BTree.get_value(node_idx: Int, i: Int) -> Int {
  if i == 0 { return nodes[node_idx].v0; }
  if i == 1 { return nodes[node_idx].v1; }
  return nodes[node_idx].v2;
}

fn BTree.set_entry(node_idx: Int, i: Int, key: Int, value: Int) {
  if i == 0 {
    nodes[node_idx].k0 = key;
    nodes[node_idx].v0 = value;
  } elif i == 1 {
    nodes[node_idx].k1 = key;
    nodes[node_idx].v1 = value;
  } else {
    nodes[node_idx].k2 = key;
    nodes[node_idx].v2 = value;
  }
}

fn BTree.copy_entry(node_idx: Int, from_i: Int, to_i: Int) {
  var k: Int = 0;
  var v: Int = 0;
  if from_i == 0 { k = nodes[node_idx].k0; v = nodes[node_idx].v0; }
  elif from_i == 1 { k = nodes[node_idx].k1; v = nodes[node_idx].v1; }
  else { k = nodes[node_idx].k2; v = nodes[node_idx].v2; }
  if to_i == 0 { nodes[node_idx].k0 = k; nodes[node_idx].v0 = v; }
  elif to_i == 1 { nodes[node_idx].k1 = k; nodes[node_idx].v1 = v; }
  else { nodes[node_idx].k2 = k; nodes[node_idx].v2 = v; }
}

fn alloc_node(tree: &mut BTree) -> Int {
  if tree.num_nodes >= NODE_POOL_SIZE {
    return -1;
  }
  let idx: Int = tree.num_nodes;
  var n = BTreeNode.new_leaf();
  tree.nodes.push(n);
  tree.num_nodes = tree.num_nodes + 1;
  return idx;
}

fn BTree.search(node_idx: Int, key: Int) -> Option[Int] {
  var nk: Int = nodes[node_idx].num_keys;
  var i: Int = 0;
  while i < nk && key > get_key(node_idx, i) {
    i = i + 1;
  }
  if i < nk && key == get_key(node_idx, i) {
    return Some(get_value(node_idx, i));
  }
  return None;
}

fn BTree.insert_nonfull(node_idx: Int, key: Int, value: Int) {
  if nodes[node_idx].num_keys >= 3 { return; }
  var i: Int = nodes[node_idx].num_keys - 1;
  while i >= 0 && key < get_key(node_idx, i) {
    copy_entry(node_idx, i, i + 1);
    i = i - 1;
  }
  set_entry(node_idx, i + 1, key, value);
  nodes[node_idx].num_keys = nodes[node_idx].num_keys + 1;
}

fn main() {
  var tree = BTree{
    root_offset: 0,
    num_nodes: 0,
    nodes: Vec[BTreeNode].with_capacity(NODE_POOL_SIZE)
  };
  let root = alloc_node(&mut tree);
  tree.root_offset = root;

  var i: Int = 0;
  while i < 50000 {
    tree.insert_nonfull(root, i * 3, i * 7);
    i = i + 1;
  }

  var found: Int = 0;
  i = 0;
  while i < 20000 {
    match tree.search(root, i * 5) {
      Some(v) => {
        found = found + 1;
      }
      None => {}
    }
    i = i + 1;
  }

  io.println("OK");
}
