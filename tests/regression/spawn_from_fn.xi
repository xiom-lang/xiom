// E2E: Spawn from helper fn with captured params
use xiom.io;

fn launch_task(x: Int) {
  spawn move {
    var result = x * 2;
    if result == 200 { io.println("PASS"); }
  }
}

fn main() -> Int {
  launch_task(100);
  return 0;
}
