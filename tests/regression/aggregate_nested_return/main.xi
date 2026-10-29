// REL-20261029-1324: a multi-level aggregate return must transfer the Vec
// fields of EVERY aggregate level to the caller. `Store{..} -> Tree{ store:
// store } -> Some(Snapshot{ tree: tree })` shallow-copies the header at each
// level; if an intermediate aggregate local stays in the drop set, its Vec
// buffers are freed while the returned value still points at them (the 0.2.1
// orbitdb carve crashed at 82 passes in `snap_load_v2`). Each check reads
// through the returned tree after a same-size allocation churn, so the freed
// buffers are reused and stale values are observed without ASan.
module aggregate_nested_return

use xiom.io;
use xiom.convert;

type Node = { keys: Vec[Int]; }
type Store = { nodes: Vec[Node]; }
type Tree = { store: Store; order: Int; }
type Snap = { tree: Tree; lsn: Int; }

fn make_node(n: Int) -> Node {
  var keys = Vec[Int].new();
  keys.push(n);
  return Node{ keys: keys };
}

fn churn_nodes() -> Int {
  var nodes = Vec[Node].new();
  var i = 0;
  while i < 8 {
    nodes.push(make_node(9000 + i));
    i = i + 1;
  }
  return nodes[0].keys[0];
}

// Two aggregate levels, direct return.
fn make_two_levels(n: Int) -> Snap {
  var nodes = Vec[Node].new();
  var i = 0;
  while i < n {
    nodes.push(make_node(i));
    i = i + 1;
  }
  var store = Store{ nodes: nodes };
  var tree = Tree{ store: store, order: 3 };
  return Snap{ tree: tree, lsn: 42 };
}

// One aggregate level behind an Option (sret) return.
fn make_option_one(n: Int) -> Option[Tree] {
  var nodes = Vec[Node].new();
  var i = 0;
  while i < n {
    nodes.push(make_node(i));
    i = i + 1;
  }
  var store = Store{ nodes: nodes };
  var tree = Tree{ store: store, order: 3 };
  return Some(tree);
}

// The carve shape: two levels behind an Option.
fn make_option_two(n: Int) -> Option[Snap] {
  var nodes = Vec[Node].new();
  var i = 0;
  while i < n {
    nodes.push(make_node(i));
    i = i + 1;
  }
  var store = Store{ nodes: nodes };
  var tree = Tree{ store: store, order: 3 };
  return Some(Snap{ tree: tree, lsn: 42 });
}

// Control: one aggregate level, direct return.
fn make_direct(n: Int) -> Store {
  var nodes = Vec[Node].new();
  var i = 0;
  while i < n {
    nodes.push(make_node(i));
    i = i + 1;
  }
  return Store{ nodes: nodes };
}

fn main() -> Int {
  var a = make_two_levels(8);
  let _c1 = churn_nodes();
  if a.tree.store.nodes.len() != 8 { return 1; }
  if a.tree.store.nodes[0].keys[0] != 0 { return 2; }
  if a.tree.store.nodes[7].keys[0] != 7 { return 3; }
  if a.tree.order != 3 || a.lsn != 42 { return 4; }

  match make_option_one(8) {
    Some(t) => {
      let _c2 = churn_nodes();
      if t.store.nodes.len() != 8 { return 5; }
      if t.store.nodes[0].keys[0] != 0 { return 6; }
      if t.store.nodes[7].keys[0] != 7 { return 7; }
    }
    None => { return 8; }
  }

  match make_option_two(8) {
    Some(s) => {
      let _c3 = churn_nodes();
      if s.tree.store.nodes.len() != 8 { return 9; }
      if s.tree.store.nodes[0].keys[0] != 0 { return 10; }
      if s.tree.store.nodes[7].keys[0] != 7 { return 11; }
    }
    None => { return 12; }
  }

  var d = make_direct(8);
  let _c4 = churn_nodes();
  if d.nodes.len() != 8 { return 13; }
  if d.nodes[0].keys[0] != 0 { return 14; }

  io.println("aggregate nested return ok");
  return 0;
}
