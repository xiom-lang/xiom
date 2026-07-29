module regression.m18_guard_0056

fn main() -> Int {
  var r: Result[Int, Str] = Ok(42);
  match r {
    Ok(v) => {
      match v {
        n if n > 10 => { return 0; }
        _ => { return 1; }
      }
    }
    _ => { return 2; }
  }
}
