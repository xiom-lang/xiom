module repro
use xiom.io;
use xiom.test;

fn main() -> Int {
  let r = assert(1 == 1, "one equals one");
  io.println((if r.passed { "T" } else { "F" }) + "/" + r.name);
  return 0;
}
