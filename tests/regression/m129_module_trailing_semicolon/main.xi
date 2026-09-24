// m129 (FE-17): a trailing semicolon after a brace-less `module x;` header is
// tolerated. Before the fix the parser consumed only `module x`, and `;` fell
// through to the top-level parser as the spurious error
// "expected declaration, found ';'" (P001), so packages written with the
// semicolon could not build.
module m129_module_trailing_semicolon;

fn main() -> Int {
  return 0;
}
