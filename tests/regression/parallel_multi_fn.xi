// E2E: Parallel codegen — multiple independent functions
use xiom.io;

fn f1() -> Int { return 1; }
fn f2() -> Int { return 2; }
fn f3() -> Int { return 3; }
fn f4() -> Int { return 4; }
fn f5() -> Int { return f1() + f2() + f3() + f4(); }

fn main() -> Int {
  var result = f5();
  if result != 10 { return 1; }
  io.println("PASS: parallel codegen multi-fn");
  return 0;
}
