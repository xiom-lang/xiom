module regression.m18_guard_0105

fn main() -> Int {
  var v: Int = 5;
  match v {
    v if v as Float64 > 3.0 => { return 0; }
    _ => { return 1; }
  }
}
