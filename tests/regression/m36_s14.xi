// M36-S14: AST manipulation — map (transform) each node in an expression tree
type MapNode = { kind: Int; value: Int; transformed: Bool; }
fn make_map_node(k: Int, v: Int) -> MapNode {
  return MapNode{ kind: k; value: v; transformed: false; };
}
fn map_constant_fold(node: MapNode) -> MapNode {
  if node.kind == 5 {
    return MapNode{ kind: node.kind; value: node.value * 2; transformed: true; };
  }
  return MapNode{ kind: node.kind; value: node.value; transformed: false; };
}
fn map_increment_all(node: MapNode) -> MapNode {
  return MapNode{ kind: node.kind; value: node.value + 1; transformed: true; };
}
fn map_to_identity(node: MapNode) -> MapNode {
  if node.kind == 0 { return MapNode{ kind: node.kind; value: 0; transformed: true; }; }
  return node;
}
fn count_transformed(n1: Bool, n2: Bool, n3: Bool) -> Int {
  var c: Int = 0;
  if n1 { c = c + 1; }
  if n2 { c = c + 1; }
  if n3 { c = c + 1; }
  return c;
}
fn main() -> Int {
  var n1 = make_map_node(1, 10);
  var n2 = make_map_node(5, 20);
  var n3 = make_map_node(0, 30);
  var m1 = map_constant_fold(n1);
  var m2 = map_constant_fold(n2);
  var m3 = map_constant_fold(n3);
  if m1.value != 10 { return 1; }
  if m2.value != 40 { return 2; }
  if m3.value != 30 { return 3; }
  var i1 = map_increment_all(n1);
  var i2 = map_increment_all(n2);
  var i3 = map_increment_all(n3);
  if i1.value != 11 { return 4; }
  if i2.value != 21 { return 5; }
  if i3.value != 31 { return 6; }
  var z1 = map_to_identity(n1);
  var z2 = map_to_identity(n3);
  if z1.value != 10 { return 7; }
  if z2.value != 0 { return 8; }
  var ct = count_transformed(m2.transformed, z1.transformed, i3.transformed);
  if ct != 2 { return 9; }
  return 0;
}
