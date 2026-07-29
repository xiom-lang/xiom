module regression.m18_guard_0049

enum Msg {
  Start
  Data(v: Int)
  Stop
}

fn main() -> Int {
  var m: Msg = Data(42);
  match m {
    Data(v) if v > 0 => { return 0; }
    _ => { return 1; }
  }
}
