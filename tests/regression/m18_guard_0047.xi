module regression.m18_guard_0047

enum Value {
  None
  Int32(n: Int)
}

fn main() -> Int {
  var v: Value = Int32(42);
  match v {
    Int32(n) if n > 100 => { return 1; }
    _ => { return 0; }
  }
}
