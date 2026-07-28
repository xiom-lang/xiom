// M36-S12: AST manipulation — rewrite node by replacing children
type AstTag = { node_type: Int; flags: Int; }
type AstChild = { index: Int; node_id: Int; }
type AstRewrite = { tag: AstTag; old_child: AstChild; new_child: AstChild; }
fn make_tag(nt: Int, flags: Int) -> AstTag {
  return AstTag{ node_type: nt; flags: flags; };
}
fn make_child(idx: Int, nid: Int) -> AstChild {
  return AstChild{ index: idx; node_id: nid; };
}
fn make_rewrite(tag: AstTag, old: AstChild, new: AstChild) -> AstRewrite {
  return AstRewrite{ tag: tag; old_child: old; new_child: new; };
}
fn is_binary_expr(tag: AstTag) -> Bool {
  return tag.node_type >= 10 && tag.node_type <= 19;
}
fn is_literal(tag: AstTag) -> Bool {
  return tag.node_type == 1;
}
fn replace_child(r: AstRewrite) -> Int {
  if r.old_child.index == r.new_child.index { return r.new_child.node_id; }
  return r.old_child.node_id;
}
fn child_count(r: AstRewrite) -> Int {
  return r.old_child.index + r.new_child.index + r.tag.flags;
}
fn main() -> Int {
  var add_tag = make_tag(10, 0);
  var lit_tag = make_tag(1, 0);
  if !is_binary_expr(add_tag) { return 1; }
  if !is_literal(lit_tag) { return 2; }
  if is_literal(add_tag) { return 3; }
  var old_c = make_child(0, 42);
  var new_c = make_child(0, 99);
  var rw = make_rewrite(add_tag, old_c, new_c);
  var result = replace_child(rw);
  if result != 99 { return 4; }
  var rw2 = make_rewrite(add_tag, make_child(1, 10), make_child(0, 20));
  var result2 = replace_child(rw2);
  if result2 != 10 { return 5; }
  if child_count(rw) != 0 { return 6; }
  return 0;
}
