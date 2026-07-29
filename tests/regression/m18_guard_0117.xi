module regression.m18_guard_0117

fn main() -> Int {
  var res: Result[Int, Str] = Ok(42);
  match res {
    Ok(v) if v > 5 => {
      match v {
        n if n == 42 => { return 0; }
        _ => { return 1; }
      }
    }
    _ => { return 2; }
  }
}
