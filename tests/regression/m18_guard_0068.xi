module regression.m18_guard_0068

fn main() -> Int {
  var x: Int32 = 100000i32;
  match x {
    v if v > 50000i32 => { return 0; }
    _ => { return 1; }
  }
}
