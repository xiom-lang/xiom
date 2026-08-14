use repro_map_keys;

fn main() -> Int {
  record("a");
  record("b");
  var n = keys_count();
  if n != 2 { return 1; }
  return 0;
}
