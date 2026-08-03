// I1: Send enforcement — Int capture (Send, must pass)
use xiom.io;

fn main() -> Int {
  var x: Int = 100;
  spawn move {
    var result = x + 1;
    if result == 101 {
      io.println("PASS: Int is Send");
    }
  }
  return 0;
}
