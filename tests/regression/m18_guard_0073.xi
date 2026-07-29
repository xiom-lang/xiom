module regression.m18_guard_0073

fn main() -> Int {
  var flag: Bool = true;
  match flag {
    v if v => { return 0; }
    _ => { return 1; }
  }
}
