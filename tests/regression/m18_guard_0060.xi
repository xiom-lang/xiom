module regression.m18_guard_0060

enum Status {
  Ok
  Error(code: Int)
}

fn main() -> Int {
  var opt = Some(Error(500));
  match opt {
    Some(s) => {
      match s {
        Error(c) if c == 404 => { return 1; }
        Error(c) => { return 0; }
        _ => { return 2; }
      }
    }
    None => { return 3; }
  }
}
