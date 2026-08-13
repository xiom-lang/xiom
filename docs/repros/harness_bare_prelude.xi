use repro_bare_prelude;

fn main() -> Int {
  var s = to_str(42);
  if s != "42" { return 1; }
  return 0;
}
