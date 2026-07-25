// M16: Hello World compiles with ZERO warnings (was 5 warnings before fix)
// Regression test for: Vec[UInt8], generic T, Self warnings
use xiom.io;

fn main() {
  io.println("Hello, XIOM!");
}
