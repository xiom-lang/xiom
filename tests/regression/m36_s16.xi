// M36-S16: Pretty printer — advanced formatting with alignment rules
type PPStyle = { spaces_per_indent: Int; max_width: Int; use_tabs: Bool; }
type PPLine = { text: Str; depth: Int; align_col: Int; }
fn make_style(spi: Int, mw: Int, tabs: Bool) -> PPStyle {
  return PPStyle{ spaces_per_indent: spi; max_width: mw; use_tabs: tabs; };
}
fn make_line(text: Str, depth: Int, align: Int) -> PPLine {
  return PPLine{ text: text; depth: depth; align_col: align; };
}
fn indent_width(style: PPStyle, depth: Int) -> Int {
  return style.spaces_per_indent * depth;
}
fn fits_in_width(style: PPStyle, line: PPLine) -> Bool {
  return (indent_width(style, line.depth) + line.text.len()) <= style.max_width;
}
fn needs_break(style: PPStyle, line: PPLine) -> Bool {
  return !fits_in_width(style, line);
}
fn check_alignment(line: PPLine, expected_align: Int) -> Bool {
  return line.align_col == expected_align;
}
fn main() -> Int {
  var s4 = make_style(4, 80, false);
  var s2 = make_style(2, 40, true);
  if s4.spaces_per_indent != 4 || s4.max_width != 80 { return 1; }
  if s2.use_tabs != true { return 2; }
  var short = make_line("var x = 1;", 2, 0);
  var long = make_line("var very_long_variable_name = some_function_call(a, b, c);", 3, 0);
  if indent_width(s4, 2) != 8 { return 3; }
  if indent_width(s2, 3) != 6 { return 4; }
  if !fits_in_width(s4, short) { return 5; }
  if needs_break(s4, short) { return 6; }
  var styled = make_line("x + y", 1, 4);
  if !check_alignment(styled, 4) { return 7; }
  if check_alignment(styled, 5) { return 8; }
  return 0;
}
