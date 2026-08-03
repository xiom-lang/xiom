// I1: Send enforcement — struct with Send fields must pass
use xiom.io;

type Point = {
  x: Int;
  y: Int;
}

fn main() -> Int {
  var p = Point{ x: 10, y: 20 };
  spawn move {
    io.println("PASS: struct Point is Send");
  }
  return 0;
}
