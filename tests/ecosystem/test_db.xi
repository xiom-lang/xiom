// XIOM — Database Ecosystem Compiler Hardening Test
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module tests.ecosystem.test_db
use xiom.collections;

pub type BTreeNode = {
  keys: Vec[Int];
  values: Vec[Int];
  children: Vec[Int];
  is_leaf: Bool;
}

pub type BTree = {
  nodes: Vec[BTreeNode];
  root_idx: Int;
  order: Int;
}

pub type WALEntry = {
  op: Int;
  key: Int;
  value: Int;
  timestamp: Int;
}

pub type WAL = {
  entries: Vec[WALEntry];
}

// ============================================================================
// B-Tree Node Operations
// ============================================================================

fn btree_node_new(is_leaf: Bool) -> BTreeNode {
  return BTreeNode{
    keys: Vec[Int].new(),
    values: Vec[Int].new(),
    children: Vec[Int].new(),
    is_leaf: is_leaf,
  };
}

// ============================================================================
// B-Tree Operations
// ============================================================================

fn btree_new(order: Int) -> BTree {
  var nodes = Vec[BTreeNode].new();
  var root = btree_node_new(true);
  nodes.push(root);
  return BTree{ nodes: nodes, root_idx: 0, order: order };
}

fn btree_insert(tree: &mut BTree, key: Int, value: Int) -> Bool {
  var idx = tree.root_idx;
  var node = tree.nodes[idx];
  while node.is_leaf == false {
    var children = node.children;
    if children.len() == 0 { return false; }
    var pos = 0;
    while pos < node.keys.len() {
      if key < node.keys[pos] { break; }
      pos = pos + 1;
    }
    if pos >= children.len() { return false; }
    idx = children[pos];
    node = tree.nodes[idx];
  }
  var ins = 0;
  while ins < node.keys.len() {
    if node.keys[ins] == key {
      tree.nodes[idx].values[ins] = value;
      return true;
    }
    if key < node.keys[ins] { break; }
    ins = ins + 1;
  }
  tree.nodes[idx].keys.insert(ins, key);
  tree.nodes[idx].values.insert(ins, value);
  return true;
}

fn btree_search(tree: &BTree, key: Int) -> Option[Int] {
  var idx = tree.root_idx;
  var node = tree.nodes[idx];
  while node.is_leaf == false {
    var children = node.children;
    if children.len() == 0 { return None; }
    var pos = 0;
    while pos < node.keys.len() {
      if key < node.keys[pos] { break; }
      pos = pos + 1;
    }
    if pos >= children.len() { return None; }
    idx = children[pos];
    node = tree.nodes[idx];
  }
  var i = 0;
  while i < node.keys.len() {
    if node.keys[i] == key { return Some(node.values[i]); }
    i = i + 1;
  }
  return None;
}

fn btree_len(tree: &BTree) -> Int {
  var idx = tree.root_idx;
  var node = tree.nodes[idx];
  while node.is_leaf == false {
    var children = node.children;
    if children.len() == 0 { return 0; }
    idx = children[0];
    node = tree.nodes[idx];
  }
  return node.keys.len();
}

// ============================================================================
// WAL Operations
// ============================================================================

fn wal_new() -> WAL {
  return WAL{ entries: Vec[WALEntry].new() };
}

fn wal_append(wal: &mut WAL, entry: WALEntry) {
  wal.entries.push(entry);
}

fn wal_len(wal: &WAL) -> Int {
  return wal.entries.len();
}

// ============================================================================
// Test Functions
// ============================================================================

fn test_btree_new_empty() -> Bool {
  var tree = btree_new(5);
  var node = tree.nodes[0];
  if node.is_leaf == true { return true; }
  return false;
}

fn test_btree_node_new_leaf() -> Bool {
  var node = btree_node_new(true);
  if node.is_leaf && node.keys.len() == 0 && node.values.len() == 0 { return true; }
  return false;
}

fn test_btree_node_new_internal() -> Bool {
  var node = btree_node_new(false);
  if node.is_leaf == false { return true; }
  return false;
}

fn test_btree_insert_one() -> Bool {
  var tree = btree_new(3);
  var ok = btree_insert(&mut tree, 10, 100);
  if ok == false { return false; }
  var result = btree_search(&tree, 10);
  match result {
    Some(v) => { return v == 100; }
    None => { return false; }
  }
}

fn test_btree_insert_multiple() -> Bool {
  var tree = btree_new(3);
  btree_insert(&mut tree, 10, 100);
  btree_insert(&mut tree, 20, 200);
  btree_insert(&mut tree, 30, 300);
  var r1 = btree_search(&tree, 10);
  var r2 = btree_search(&tree, 20);
  var r3 = btree_search(&tree, 30);
  match r1 {
    Some(v1) => {
      match r2 {
        Some(v2) => {
          match r3 {
            Some(v3) => { return v1 == 100 && v2 == 200 && v3 == 300; }
            None => { return false; }
          }
        }
        None => { return false; }
      }
    }
    None => { return false; }
  }
}

fn test_btree_search_missing() -> Bool {
  var tree = btree_new(3);
  btree_insert(&mut tree, 10, 100);
  var result = btree_search(&tree, 99);
  match result {
    Some(v) => { return false; }
    None => { return true; }
  }
}

fn test_btree_update_existing() -> Bool {
  var tree = btree_new(3);
  btree_insert(&mut tree, 42, 100);
  btree_insert(&mut tree, 42, 200);
  var result = btree_search(&tree, 42);
  match result {
    Some(v) => { return v == 200; }
    None => { return false; }
  }
}

fn test_btree_len_after_inserts() -> Bool {
  var tree = btree_new(3);
  btree_insert(&mut tree, 1, 10);
  btree_insert(&mut tree, 2, 20);
  btree_insert(&mut tree, 3, 30);
  var count = btree_len(&tree);
  if count == 3 { return true; }
  return false;
}

fn test_btree_len_after_duplicates() -> Bool {
  var tree = btree_new(3);
  btree_insert(&mut tree, 1, 10);
  btree_insert(&mut tree, 1, 11);
  btree_insert(&mut tree, 2, 20);
  var count = btree_len(&tree);
  if count == 2 { return true; }
  return false;
}

fn test_wal_new_empty() -> Bool {
  var wal = wal_new();
  if wal_len(&wal) == 0 { return true; }
  return false;
}

fn test_wal_append_single() -> Bool {
  var wal = wal_new();
  var entry = WALEntry{ op: 1, key: 42, value: 100, timestamp: 1000 };
  wal_append(&mut wal, entry);
  if wal_len(&wal) == 1 { return true; }
  return false;
}

fn test_wal_append_multiple() -> Bool {
  var wal = wal_new();
  var e1 = WALEntry{ op: 1, key: 10, value: 100, timestamp: 1 };
  var e2 = WALEntry{ op: 1, key: 20, value: 200, timestamp: 2 };
  var e3 = WALEntry{ op: 2, key: 10, value: 999, timestamp: 3 };
  wal_append(&mut wal, e1);
  wal_append(&mut wal, e2);
  wal_append(&mut wal, e3);
  if wal_len(&wal) == 3 { return true; }
  return false;
}

fn test_wal_entry_fields() -> Bool {
  var entry = WALEntry{ op: 2, key: 55, value: 555, timestamp: 9999 };
  if entry.op == 2 && entry.key == 55 && entry.value == 555 && entry.timestamp == 9999 { return true; }
  return false;
}

fn test_btree_with_wal_ops() -> Bool {
  var tree = btree_new(4);
  var wal = wal_new();
  btree_insert(&mut tree, 5, 50);
  var e1 = WALEntry{ op: 1, key: 5, value: 50, timestamp: 1 };
  wal_append(&mut wal, e1);
  btree_insert(&mut tree, 8, 80);
  var e2 = WALEntry{ op: 1, key: 8, value: 80, timestamp: 2 };
  wal_append(&mut wal, e2);
  btree_insert(&mut tree, 5, 55);
  var e3 = WALEntry{ op: 2, key: 5, value: 55, timestamp: 3 };
  wal_append(&mut wal, e3);
  var found = btree_search(&tree, 5);
  match found {
    Some(v) => {
      if v == 55 && wal_len(&wal) == 3 { return true; }
      return false;
    }
    None => { return false; }
  }
  return false;
}

fn test_btree_many_inserts_ascending() -> Bool {
  var tree = btree_new(4);
  var i = 0;
  while i < 20 {
    btree_insert(&mut tree, i, i * 10);
    i = i + 1;
  }
  var r0 = btree_search(&tree, 0);
  var r9 = btree_search(&tree, 9);
  var r19 = btree_search(&tree, 19);
  var r99 = btree_search(&tree, 99);
  match r0 {
    Some(v0) => {
      match r9 {
        Some(v9) => {
          match r19 {
            Some(v19) => {
              match r99 {
                Some(v99) => { return false; }
                None => { return v0 == 0 && v9 == 90 && v19 == 190; }
              }
            }
            None => { return false; }
          }
        }
        None => { return false; }
      }
    }
    None => { return false; }
  }
}

fn test_btree_many_inserts_descending() -> Bool {
  var tree = btree_new(4);
  var i = 19;
  while i >= 0 {
    btree_insert(&mut tree, i, i * 100);
    i = i - 1;
  }
  var r0 = btree_search(&tree, 0);
  var r19 = btree_search(&tree, 19);
  match r0 {
    Some(v0) => {
      match r19 {
        Some(v19) => { return v0 == 0 && v19 == 1900; }
        None => { return false; }
      }
    }
    None => { return false; }
  }
}

fn test_btree_order_stored() -> Bool {
  var tree = btree_new(7);
  if tree.order == 7 { return true; }
  return false;
}

fn test_wal_preserves_order() -> Bool {
  var wal = wal_new();
  var i = 0;
  while i < 10 {
    var e = WALEntry{ op: 1, key: i, value: i * 10, timestamp: i };
    wal_append(&mut wal, e);
    i = i + 1;
  }
  var ok = true;
  var j = 0;
  while j < wal.entries.len() {
    if wal.entries[j].key != j { ok = false; }
    j = j + 1;
  }
  return ok;
}

// ============================================================================
// Main
// ============================================================================

fn main() -> Int {
  var failed = false;
  if test_btree_new_empty() == false { failed = true; }
  if test_btree_node_new_leaf() == false { failed = true; }
  if test_btree_node_new_internal() == false { failed = true; }
  if test_btree_insert_one() == false { failed = true; }
  if test_btree_insert_multiple() == false { failed = true; }
  if test_btree_search_missing() == false { failed = true; }
  if test_btree_update_existing() == false { failed = true; }
  if test_btree_len_after_inserts() == false { failed = true; }
  if test_btree_len_after_duplicates() == false { failed = true; }
  if test_wal_new_empty() == false { failed = true; }
  if test_wal_append_single() == false { failed = true; }
  if test_wal_append_multiple() == false { failed = true; }
  if test_wal_entry_fields() == false { failed = true; }
  if test_btree_with_wal_ops() == false { failed = true; }
  if test_btree_many_inserts_ascending() == false { failed = true; }
  if test_btree_many_inserts_descending() == false { failed = true; }
  if test_btree_order_stored() == false { failed = true; }
  if test_wal_preserves_order() == false { failed = true; }
  if failed { return 1; }
  return 0;
}
