module m252_cache_main;

use m252_helper;

fn main() -> Int {
  if m252_helper.triple(4) != 48 {
    return 1;
  }
  return 0;
}
