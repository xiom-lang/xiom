use xiom.io;

fn main() -> Int {
  var x: Int = 100;
  spawn move {
    io.println("spawned");
    var result = x + 1;
    if result != 101 {
      io.println("FAIL");
    } else {
      io.println("PASS: spawn captured x=100");
    }
  }
  return 0;
}
