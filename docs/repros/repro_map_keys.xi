module repro_map_keys

use xiom.collections;

var _coverage: Map[Str, Bool] = Map[Str, Bool].new();

pub fn record(k: Str) {
  _coverage.insert(k, true);
}

pub fn keys_count() -> Int {
  let keys = _coverage.keys();
  return keys.len();
}
