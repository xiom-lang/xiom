// M36-S15: Pretty printer -- indentation and prefix-based formatting
type PPDoc = { indent: Int; content: Str; is_break: Bool; }
fn make_doc(indent: Int, content: Str, br: Bool) -> PPDoc {
  return PPDoc{ indent: indent; content: content; is_break: br; };
}
fn increase_indent(doc: PPDoc, delta: Int) -> PPDoc {
  return PPDoc{ indent: doc.indent + delta; content: doc.content; is_break: doc.is_break; };
}
fn is_blank_line(doc: PPDoc) -> Bool {
  return doc.content.len() == 0 && doc.is_break;
}
fn is_group_start(doc: PPDoc) -> Bool {
  return doc.indent == 0 && !doc.is_break;
}
fn pretty_depth(doc: PPDoc) -> Int {
  return doc.indent * 2;
}
fn concat_indent(base: Int, extra: Int) -> Int {
  return base + extra;
}
fn main() -> Int {
  var d1 = make_doc(0, "fn main() -> Int {", false);
  var d2 = make_doc(1, "var x: Int = 0;", false);
  var d3 = make_doc(1, "return x;", false);
  var d4 = make_doc(0, "}", false);
  if d1.indent != 0 || d2.indent != 1 { return 1; }
  if d3.indent != 1 || d4.indent != 0 { return 2; }
  var deeper = increase_indent(d2, 1);
  if deeper.indent != 2 { return 3; }
  var blank = make_doc(0, "", true);
  if !is_blank_line(blank) { return 4; }
  if is_blank_line(d1) { return 5; }
  if !is_group_start(d1) { return 6; }
  if is_group_start(d2) { return 7; }
  if pretty_depth(d3) != 2 { return 8; }
  if concat_indent(d1.indent, d3.indent) != 1 { return 9; }
  return 0;
}
