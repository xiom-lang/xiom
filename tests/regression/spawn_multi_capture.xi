// E2E: Spawn with multiple captures (Int + Int + Int)
use xiom.io;

fn main() -> Int {
  var a: Int = 10;
  var b: Int = 20;
  var c: Int = 30;
  spawn move {
    var sum = a + b + c;
    if sum == 60 {
      io.println("PASS: multi-capture spawn");
    }
  }
  return 0;
}
