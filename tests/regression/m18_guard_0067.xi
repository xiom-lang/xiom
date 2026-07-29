module regression.m18_guard_0067

fn main() -> Int {
  var x: Int16 = 1000i16;
  match x {
    v if v > 500i16 => { return 0; }
    _ => { return 1; }
  }
}
