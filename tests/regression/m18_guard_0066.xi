module regression.m18_guard_0066

fn main() -> Int {
  var x: Int8 = 50i8;
  match x {
    v if v > 40i8 => { return 0; }
    _ => { return 1; }
  }
}
