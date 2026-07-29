module regression.m18_guard_0080

fn main() -> Int {
  var v: Int = 5;
  match v {
    1 | _ if true => { return 0; }
    _ => { return 1; }
  }
}
