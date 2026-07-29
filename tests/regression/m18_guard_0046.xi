module regression.m18_guard_0046

enum Value {
  None
  Int32(n: Int)
}

fn main() -> Int {
  var v: Value = Int32(42);
  match v {
    Int32(n) if n > 10 => { return 0; }
    _ => { return 1; }
  }
}
