// m154 (Stage 6 W004) negative: guarded duplicates and guarded catch-alls
// are reachable and must stay silent.
module p_w004_guard;

use xiom.io;

fn f(n: Int, c: Bool) -> Int {
  match n {
    1 if c => { 10 },
    1 => { 11 },
    _ if n < 0 => { 12 },
    _ => { 13 },
  }
}

fn main() -> Int {
  io.println(f(1, true).to_str());
  io.println(f(2, false).to_str());
  return 0;
}
