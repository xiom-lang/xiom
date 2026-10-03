// Phase 3 catalog gate, accept case: in-program modules resolve qualified
// calls through their registered exports.
module pinprog {
  pub fn bump(x: Int) -> Int {
    return x + 1;
  }
}

fn main() -> Int {
  return pinprog.bump(1);
}
