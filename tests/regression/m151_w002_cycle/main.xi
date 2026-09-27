// m151 (Stage 6 W002): a mutual-recursion cycle whose members all reach the
// cycle call before any exit must warn; compilation stays exit 0 and the
// warning text names the cycle. Do NOT execute this fixture (it can never
// terminate); the lock only compiles it.
module m151_w002_cycle;

use xiom.io;

fn ping_a() { ping_b(); io.println("A"); }
fn ping_b() { ping_a(); io.println("B"); }

fn main() -> Int {
  ping_a();
  return 0;
}
