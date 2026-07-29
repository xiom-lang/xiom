module regression.m18_guard_0121

fn main() -> Int {
  var v: Int = 1;
  match v {
    v if ((((((((((((((v > 0)))))))))))))) => { return 0; }
    _ => { return 1; }
  }
}
