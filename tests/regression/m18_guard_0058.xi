module regression.m18_guard_0058

fn main() -> Int {
  var r: Result[Int, Str] = Ok(42);
  match r {
    Ok(v) if v > 100 => { return 1; }
    Ok(_) => { return 0; }
    Err(_) => { return 2; }
  }
}
