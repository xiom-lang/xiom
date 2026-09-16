// m79 (Stage 5): DWARF for .xi. `-g` must emit a VALID debug-info graph
// (DISubroutineType was missing -- `type: !{}` made LLVM warn "ignoring
// invalid debug info" and drop all DWARF) plus per-statement DILocations so
// debuggers can bind .xi breakpoints to body lines. The e2e test asserts
// the metadata shape and that the -g build still runs.
module m79_debug_info

fn add(a: Int, b: Int) -> Int {
  return a + b;
}

fn main() -> Int {
  var x = add(2, 3);
  if x != 5 { return 1; }
  return 0;
}
